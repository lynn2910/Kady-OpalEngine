//! Error's IDs: 01xxx



use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use regex::Regex;
use serde::{Serialize, Deserialize };
use sqlx::MySqlPool;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use client::manager::events::Context;
use client::manager::http::Http;
use client::models::channel::ChannelId;
use client::models::guild::GuildId;
use client::models::message::MessageBuilder;
use client::models::user::{User, UserId};
use client::typemap::Type;
use database::dynamic_requests::DynamicRequest;
use database::model::guild::GuildUserXp;
use translation::message;



/// The default xp cooldown (in seconds)
const DEFAULT_XP_COOLDOWN: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct XpCooldownContainer {
	/// Contain as key a guild & user IDs pair, and as the value, the next time a user will be able to gain xp again
	users: Arc<RwLock<HashMap<(GuildId, UserId), Instant>>>,
	#[allow(dead_code)]
	cleaner: Arc<JoinHandle<()>>
}

impl Type for XpCooldownContainer {
	type Value = Self;
}

impl XpCooldownContainer {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		let m = Arc::new(RwLock::new(HashMap::new()));

		Self {
			cleaner: Arc::new(Self::cleaner(m.clone())),
			users: m
		}
	}

	fn cleaner(map: Arc<RwLock<HashMap<(GuildId, UserId), Instant>>>) -> JoinHandle<()> {
		let map_clone = map.clone();
		tokio::spawn(async move {
			loop {
				// every hours
				sleep(Duration::from_secs(60 * 60)).await;

				{
					let mut users = map_clone.write().await;
					users.retain(|_, instant| Instant::now().duration_since(*instant) < Duration::from_secs(0));
				}
			}
		})
	}

	/// Get the last registered instant when a user had his xp upgraded
	pub async fn get_last_user_instant(&self, key: &(GuildId, UserId)) -> Option<Instant> {
		let users = self.users.read().await;

		users.get(key).cloned()
	}

	/// Return a boolean to check if the user can get extra xp
	pub async fn is_user_ok(&self, key: &(GuildId, UserId), cooldown: Duration) -> bool {
		if let Some(u) = self.get_last_user_instant(key).await {
			u.elapsed() > cooldown
		} else {
			true
		}
	}

	/// Update the user instant
	pub async fn update_cooldown(&mut self, key: &(GuildId, UserId), cooldown: Duration) {
		let mut users = self.users.write().await;

		let _ = users.insert(key.clone(), Instant::now() + cooldown);
	}
}









#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum AlgorithmsSuites {
	#[default]
	Default = 0,
	Low = 1
}

impl From<u64> for AlgorithmsSuites {
	fn from(value: u64) -> Self {
		match value {
			1 => Self::Low,
			_ => Self::Default,
		}
	}
}

/// Calculate the level for a given xp based on the algorithm chosen
pub fn calc_xp(suite: AlgorithmsSuites, x: f64) -> f64 {
	match suite {
		AlgorithmsSuites::Default => (5.0/6.0) * (2.0 * x * x + 27.0 * x + 91.0) - (455.0 / 6.0),
		AlgorithmsSuites::Low => (2.0 * x * x) + (27.0 * x)
	}
}

/// Calculate the level for a given amount of xp
pub fn calc_lvl(suite: AlgorithmsSuites, xp: f64) -> f64 {
	let mut lvl: f64 = 0.0;
	while xp >= calc_xp(suite, lvl + 1.0) {
		lvl += 1.0;
	};
	lvl
}

/// Generate a random xp amount between 5 and 25
pub fn gen_random_xp() -> u64 {
	let mut rng = thread_rng();
	5 + (0..20).choose(&mut rng).unwrap_or(0)
}

/// This function is called for updating the xp
///
/// Don't call it when the channel is a Dm channel !!
pub async fn trigger(
	ctx: &Context,
	guild_data: &database::model::guild::Guild,
	pool: &MySqlPool,
	requests: &DynamicRequest,
	user: &User,
	channel_id: &ChannelId
) -> Result<(), u8> {
	// check if the system is enabled
	if !guild_data.xp_enabled.unwrap_or(false) || user.bot.unwrap_or(false) {
		return Ok(());
	}

	let cooldown = match guild_data.xp_cooldown {
		Some(c) => Duration::from_secs(c),
		None => DEFAULT_XP_COOLDOWN
	};

	// check the cooldown
	{
		let xp_container = if let Some(xc) = ctx.get_data::<XpCooldownContainer>().await {
			xc
		} else {
			return Err(1)
		};

		if !xp_container.is_user_ok(&(guild_data.id.clone(), user.id.clone()), cooldown).await {
			return Ok(());
		}
	}




	// ensure that he is registered
	{
		match GuildUserXp::ensure(pool, requests.guilds.xp.ensure.as_str(), &guild_data.id, &user.id).await {
			Ok(_) => (),
			Err(_) => return Err(2)
		};
	}

	let xp_data = match GuildUserXp::from_pool(pool, requests.guilds.xp.get.as_str(), &guild_data.id, &user.id).await {
		Ok(d) => d,
		Err(_) => return Err(3)
	};

	let new_xp_amount = gen_random_xp();

	if GuildUserXp::add_xp(pool, requests.guilds.xp.add_xp.as_str(), &guild_data.id, &user.id, new_xp_amount).await.is_err() { return Err(4) };

	// update the cooldown
	{
		let mut data = ctx.data.write().await;
		let xp_container = data.get_mut::<XpCooldownContainer>().expect("Cannot acquire the XpCooldownContainer structure in the context, wtf...");

		xp_container.update_cooldown(&(guild_data.id.clone(), user.id.clone()), cooldown).await;
	}

	let xp_algo: AlgorithmsSuites = guild_data.xp_algo.unwrap_or(0).into();

	{
		let old_lvl = calc_lvl(xp_algo, xp_data.xp as f64);
		let new_lvl = calc_lvl(xp_algo, (xp_data.xp + new_xp_amount) as f64);

		if old_lvl != new_lvl {
			send_lvl_up_message(
				&ctx.skynet,
				guild_data,
				user,
				channel_id,
				&xp_data,
				new_lvl as u64
			).await;
		}
	}

	Ok(())
}

lazy_static! {
	static ref MSG_FORMAT_REGEX: Regex = Regex::new(r#"\{([^{}]+)\}"#).expect("Bro wtf, i can't parse this fucking regex !");
}

fn format_lvl_up_msg(
	msg: String,
	user: &User,
	xp_data: &GuildUserXp,
	lvl: u64
) -> String {
	MSG_FORMAT_REGEX.replace_all(&msg, |captures: &regex::Captures| {
		match captures.get(1) {
			Some(capture) => match capture.as_str() {
				"username" => user.global_name.clone().unwrap_or(user.username.clone()),
				"unique_name" => user.username.clone(),
				"xp" => xp_data.xp.to_string(),
				"lvl" => lvl.to_string(),
				_ => capture.as_str().to_string()
			},
			None => String::new()
		}
	}).to_string()
}

async fn send_lvl_up_message(
	http: &Http,
	guild_data: &database::model::guild::Guild,
	user: &User,
	channel_id: &ChannelId,
	xp_data: &GuildUserXp,
	lvl: u64
) {
	let msg = match guild_data.xp_message.clone() {
		Some(m) => m,
		None => message!(guild_data.lang.clone(), "features::xp::lvl_up").to_string()
	};

	let msg_builder = MessageBuilder::new()
		.set_content(
			format_lvl_up_msg(
				msg,
				user,
				xp_data,
				lvl
			)
		);

	if let Some(xp_channel) = &guild_data.xp_channel {
		let id: ChannelId = xp_channel.clone().into();
		let _ = id.send_message(http, msg_builder).await;
	} else {
		let _ = channel_id.send_message(http, msg_builder).await;
	}
}


/// This module is used to generate images for the xp system
pub mod image_gen {
	use std::sync::Arc;
	use tokio::sync::RwLock;
	use image::{ImageBuffer, Rgba, RgbaImage};
	use image::imageops::FilterType;
	use imageproc::drawing::{Canvas, draw_filled_circle_mut, draw_filled_ellipse_mut, draw_filled_rect_mut, draw_text_mut};
	use imageproc::rect::Rect;
	use log::error;
	use rusttype::{Font, Scale};
	use client::typemap::Type;
	use error::RuntimeError;
	use crate::xp::{AlgorithmsSuites, calc_lvl, calc_xp};

	const DEFAULT_GUILD_CARD_COLOR: [u8; 4] = [74, 165, 248, 255];


	/// Contain the fonts that will be used in the card system
	#[derive(Clone)]
	pub struct FontContainer {
		pub montserrat_regular: Arc<RwLock<Font<'static>>>,
		pub montserrat_bold: Arc<RwLock<Font<'static>>>
	}

	impl FontContainer {
		pub fn new() -> Option<Self> {
			let montserrat_regular_bytes = include_bytes!("../assets/Montserrat-Regular.ttf");
			let montserrat_bold_bytes = include_bytes!("../assets/Montserrat-Bold.ttf");

			let montserrat_regular = match Font::try_from_bytes(montserrat_regular_bytes) {
				Some(f) => f,
				None => return None
			};

			let montserrat_bold = match Font::try_from_bytes(montserrat_bold_bytes) {
				Some(f) => f,
				None => return None
			};

			Some(Self {
				montserrat_regular: Arc::new(RwLock::new(montserrat_regular)),
				montserrat_bold: Arc::new(RwLock::new(montserrat_bold))
			})
		}
	}

	impl Type for FontContainer {
		type Value = Self;
	}





	const CARD_WIDTH: u32 = 935;
	const CARD_HEIGHT: u32 = 284;

	/// Generate a card image based on the informations given
	///
	/// The avatar must be a PNG (because of the use of RgbaImage) and of size 256x256
	#[allow(clippy::too_many_arguments)]
	#[allow(clippy::ptr_arg)]
	pub async fn gen_guild_image(
		user_display_name: &String,
		user_avatar: Vec<u8>,
		xp: u64,
		guild_name: &String,
		rank: u32,
		rank_traduction: String,
		fonts: &FontContainer,
		xp_algo: AlgorithmsSuites
	) -> Result<RgbaImage, RuntimeError> {
		let mut img: RgbaImage = ImageBuffer::new(CARD_WIDTH, CARD_HEIGHT);

		draw_rounded_rect(
			&mut img,
				0,
				0,
			CARD_WIDTH,
			CARD_HEIGHT,
			15,
			Rgba([35, 39, 42, 255]),
		);

		match image::load(std::io::Cursor::new(user_avatar), image::ImageFormat::Png) {
			Ok(avatar) => {
				// resize the image to 154x154
				let avatar = image::imageops::resize(&avatar.to_rgba8(), 155, 155, FilterType::Lanczos3);
				// draw the image
				draw_avatar(&mut img, 79, 65, avatar);
			}
			Err(e) => return Err(RuntimeError::new(e.to_string()))
		}

		// draw the server name
		{
			let montserrat = fonts.montserrat_regular.read().await;

			draw_text(
				&mut img,
				guild_name.as_str(),
				23,
				247,
				&montserrat,
				Rgba([135, 135, 135, 255]),
				25.0
			);
		}

		// draw the user name
		{
			let montserrat = fonts.montserrat_bold.read().await;

			draw_text(
				&mut img,
				user_display_name.as_str(),
				258,
				94,
				&montserrat,
				Rgba([255, 255, 255, 255]),
				37.0
			);
		}

		let lvl = calc_lvl(xp_algo, xp as f64);

		// determine next lvl xp and the current level xp
		let this_level_xp = calc_xp(xp_algo, lvl);
		let next_lvl_xp = calc_xp(xp_algo, lvl + 1.0);

		// draw the progress bar
		{
			let y_top = 147;

			// calculate width of the filled part of the xp bar
			let bar_width = 584; // px
			let used_bar = {
				let w = if next_lvl_xp != this_level_xp {
					(((xp as f64) - this_level_xp) / (next_lvl_xp - this_level_xp)).max(0.0)
				} else {
					0.0
				};
				(w * bar_width as f64).round() as u64
			};

			// draw empty bar first
			let bar_width_u32: u32 = match bar_width.try_into() {
				Ok(w) => w,
				Err(e) => {
					error!(target: "Runtime", "An error occured while drawing the empty bar of a guild card xp: {e:#?}");
					return Err(RuntimeError::new(e).with_target("Guild_Xp_Card_Generator"))
				}
			};
			draw_rounded_rect(
				&mut img,
				258,
				y_top,
				bar_width_u32,
				33,
				16, // 33 / 2,
				Rgba([68, 71, 74, 255]),
			);

			// used_width_value should not be lesser than 33, but also should not exceed the bar_width
			let used_width = used_bar.max(33).min(bar_width);
			let used_width_u32: u32 = match used_width.try_into() {
				Ok(w) => w,
				Err(e) => {
					error!(target: "Runtime", "An error occured while drawing the used bar of a guild card xp: {e:#?}");
					return Err(RuntimeError::new(e).with_target("Guild_Xp_Card_Generator"))
				}
			};

			// then draw the filled bar
			draw_rounded_rect(
				&mut img,
				258,
				y_top,
				used_width_u32,
				33,
				16, // 33 / 2,
				Rgba(DEFAULT_GUILD_CARD_COLOR)
			);
		}

		// draw the xp indicator
		{
			let actual_xp_formatted = crate::utils::format_number(xp);
			let next_lvl_xp_formatted = crate::utils::format_number(next_lvl_xp as u64);

			let formatted = format!("{actual_xp_formatted} / {next_lvl_xp_formatted}");

			let scale = 28.0;

			let montserrat = fonts.montserrat_regular.read().await;
			let text_size = imageproc::drawing::text_size(Scale::uniform(scale), &montserrat, formatted.as_str());

			draw_text(
				&mut img,
				formatted.as_str(),
				842 - text_size.0,
				190,
				&montserrat,
				Rgba([144, 144, 144, 255]),
				scale
			);
		}

		// draw the rank
		{
			let rank_text_scale = 25.0;

			let montserrat = fonts.montserrat_regular.read().await;
			let rank_text_size = imageproc::drawing::text_size(Scale::uniform(rank_text_scale), &montserrat, rank_traduction.as_str());

			draw_text(
				&mut img,
				rank_traduction.as_str(),
				710 - rank_text_size.0,
				57 - rank_text_size.1,
				&montserrat,
				Rgba([144, 144, 144, 255]),
				rank_text_scale
			);
			drop(montserrat);

			let rank_number_scale = 45.0;
			let formatted_rank = format!("#{rank}");

			let montserrat = fonts.montserrat_bold.read().await;
			let rank_number_size = imageproc::drawing::text_size(Scale::uniform(rank_number_scale), &montserrat, formatted_rank.as_str());

			draw_text(
				&mut img,
				formatted_rank.as_str(),
				720,
				57 - rank_number_size.1,
				&montserrat,
				Rgba([255, 255, 255, 255]),
				rank_number_scale
			);
		}

		// draw the lvl indicator
		{
			let lvl_text_scale = 25.0;

			let montserrat = fonts.montserrat_regular.read().await;
			let lvl_text_size = imageproc::drawing::text_size(Scale::uniform(lvl_text_scale), &montserrat, "LVL");

			draw_text(
				&mut img,
				"LVL",
				849 - lvl_text_size.0,
				57 - lvl_text_size.1,
				&montserrat,
				Rgba(DEFAULT_GUILD_CARD_COLOR),
				lvl_text_scale
			);
			drop(montserrat);

			let lvl_number_scale = 45.0;
			let formatted_lvl = lvl.to_string();

			let montserrat = fonts.montserrat_bold.read().await;
			let lvl_number_size = imageproc::drawing::text_size(Scale::uniform(lvl_number_scale), &montserrat, formatted_lvl.as_str());

			draw_text(
				&mut img,
				formatted_lvl.as_str(),
				854,
				57 - lvl_number_size.1,
				&montserrat,
				Rgba(DEFAULT_GUILD_CARD_COLOR),
				lvl_number_scale
			);
		}

		Ok(img)
	}

	/// Draw the rounded avatar on the image with the given coordinates
	///
	/// The image must be resized to 155x155 before calling this function
	fn draw_avatar(
		canvas: &mut RgbaImage,
		x: u32,
		y: u32,
		avatar: RgbaImage
	)
	{
		let radius = 77; // 154 / 2
		let (height, width): (u32, u32) = (155, 155);

		let mask_center: (i32, i32) = (x as i32 + radius, y as i32 + radius);
		draw_filled_circle_mut(canvas, mask_center, radius, Rgba([255, 255, 255, 255]));

		// Combiner l'image originale avec le masque circulaire
		for avatar_y in 0..height {
			for avatar_x in 0..width {
				let mut pixel = *avatar.get_pixel(avatar_x, avatar_y);

				// calculate the distance between this pixel and the center of the mask
				let distance = (((avatar_x as i32 - radius).pow(2) + (avatar_y as i32 - radius).pow(2)) as f32).sqrt();

				let alpha = if distance <= (radius - 1) as f32 {
					255
				} else {
					// calculate the alpha of the pixel
					let alpha = 255.0 - ((distance - radius as f32) * 255.0);
					alpha as u8
				};

				let mask_pixel = canvas.get_pixel(x + avatar_x, y + avatar_y);
				if *mask_pixel == Rgba([255, 255, 255, 255]) {
					pixel.0[3] = alpha;
					canvas.draw_pixel(x + avatar_x, y + avatar_y, pixel);
				}
			}
		}
	}


	/// Draw a text
	fn draw_text(
		canvas: &mut RgbaImage,
		text: &str,
		x: i32,
		y: i32,
		font: &Font,
		color: Rgba<u8>,
		scale: f32,
	) {
		let scale = Scale::uniform(scale);
		draw_text_mut(canvas, color, x, y, scale, font, text)
	}

	/// Draw a rectangle with rounded corners, from the givens coordinates, color & radius
	///
	/// TODO: use another implementation of this algorithm
	fn draw_rounded_rect(
		canvas: &mut RgbaImage,
		x: i32,
		y: i32,
		width: u32,
		height: u32,
		radius: u32,
		color: Rgba<u8>
	)
	{
		// draw top to bottom rectangle
		draw_filled_rect_mut(
			canvas,
			Rect::at(x + radius as i32, y)
				.of_size(width - (radius * 2), height),
			color
		);

		// draw left to right rectangle
		draw_filled_rect_mut(
			canvas,
			Rect::at(x, y + radius as i32)
				.of_size(width, height - (radius * 2)),
			color
		);



		// draw top right circle
		draw_filled_ellipse_mut(
			canvas,
			(x + radius as i32, y + radius as i32),
			radius as i32,
			radius as i32,
			color
		);

		// draw top left circle
		draw_filled_ellipse_mut(
			canvas,
			(x + (width as i32) - radius as i32, y + radius as i32),
			radius as i32,
			radius as i32,
			color
		);

		// draw bottom right circle
		draw_filled_ellipse_mut(
			canvas,
			(x + radius as i32, y + (height as i32) - (radius as i32) - 1),
			radius as i32,
			radius as i32,
			color
		);

		// draw bottom left circle
		draw_filled_ellipse_mut(
			canvas,
			(x + (width as i32) - radius as i32, y + (height as i32) - (radius as i32) - 1),
			radius as i32,
			radius as i32,
			color
		);


		// and now the fun, blending the rectangles
		let x = x as u32;
		let y = y as u32;
		for xpos in x..(x + width) {
			for ypos in y..(y + height) {
				let px = canvas.get_pixel(xpos, ypos);

				if *px == color {
					// check if the pixel is on a circle
					if xpos <= radius && ypos <= radius {
						// top left radius
						let distance = ((xpos as f32 - radius as f32).powf(2.0) + (ypos as f32 - radius as f32).powf(2.0)).sqrt();
						if distance > radius as f32 - 1.0 {
							let alpha = 255.0 - ((distance - radius as f32) * 255.0);
							canvas.draw_pixel(
								xpos,
								ypos,
								Rgba([color.0[0], color.0[1], color.0[2], alpha as u8])
							)
						}
					} else if xpos >= x + width - radius && ypos <= radius {
						// top right radius
						let distance = ((xpos as f32 - (x + width - radius) as f32).powf(2.0) + (ypos as f32 - radius as f32).powf(2.0)).sqrt();
						if distance > radius as f32 - 1.0 {
							let alpha = 255.0 - ((distance - radius as f32) * 255.0);
							canvas.draw_pixel(
								xpos,
								ypos,
								Rgba([color.0[0], color.0[1], color.0[2], alpha as u8])
							)
						}
					} else if xpos <= radius && ypos >= y + height - radius {
						// bottom left radius
						let distance = ((xpos as f32 - radius as f32).powf(2.0) + (ypos as f32 - (y + height - radius) as f32).powf(2.0)).sqrt();
						if distance > radius as f32 - 1.0 {
							let alpha = 255.0 - ((distance - radius as f32) * 255.0);
							// px.0[3] = alpha as u8;
							canvas.draw_pixel(
								xpos,
								ypos,
								Rgba([color.0[0], color.0[1], color.0[2], alpha as u8])
							)
						}
					} else if xpos >= x + width - radius && ypos >= y + height - radius {
						// bottom right radius
						let distance = ((xpos as f32 - (x + width - radius) as f32).powf(2.0) + (ypos as f32 - (y + height - radius) as f32).powf(2.0)).sqrt();
						if distance > radius as f32 - 1.0 {
							let alpha = 255.0 - ((distance - radius as f32) * 255.0);
							canvas.draw_pixel(
								xpos,
								ypos,
								Rgba([color.0[0], color.0[1], color.0[2], alpha as u8])
							)
						}
					}
				}
			}
		}
	}

	#[cfg(test)]
	mod test_xp_image_generator {
		use std::time::Instant;
		use crate::xp::AlgorithmsSuites;
		use crate::xp::image_gen::{FontContainer, gen_guild_image};

		#[test]
		fn generate_image() {
			let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
			runtime.block_on(async move {
				let avatar = std::fs::read(r"C:\Users\cedic\Code\kady\OpalEngine\images_tests\avatar_test.png").unwrap();

				let fonts = FontContainer::new().expect("cannot load fonts");

				let now = Instant::now();

				let img = gen_guild_image(
					&"Mia".to_string(),
					avatar,
					138,
					&"Kady Support".to_string(),
					1,
					"rang".to_string().to_uppercase(),
					&fonts,
					AlgorithmsSuites::Default
				).await;

				eprintln!("image generated in {}s", now.elapsed().as_secs_f64());

				let _ = match img {
					Ok(img) => img.save(r"C:\Users\cedic\Code\kady\OpalEngine\images_tests\test.png"),
					Err(e) => panic!("{e:#?}")
				};

				todo!()
			});
		}
	}
}
