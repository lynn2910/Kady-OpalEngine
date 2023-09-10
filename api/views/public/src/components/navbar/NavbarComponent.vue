<script setup lang="ts">
import CategoryItem from "@/components/navbar/CategoryItem.vue";
import LangSelector from "@/components/home/LangSelector.vue";
</script>

<template>
  <nav>
    <router-link to="/" class="name_icon">
      <img src="/icon_round_little.png" alt="kady_avatar">
      <h1>Kady</h1>
    </router-link>

    <div class="categories">
      <CategoryItem title="Ressources" id="support_resource">
        <div class="support_resource">
          <div class="support">
            <h4>Support</h4>
            <ul>
              <li>
                <RouterLink to="">
                  <img src="@/assets/svg/icons/question_mark.svg" alt="">
                  <p class="fr">Tutoriels</p>
                  <p class="en">Tutorials</p>
                </RouterLink>
              </li>
              <li>
                <RouterLink to="">
                  <img src="@/assets/svg/icons/chat.svg" alt="">
                  <p class="fr">Assistance</p>
                  <p class="en">Help</p>
                </RouterLink>
              </li>
              <li>
                <a href="https://discord.gg/GegJXchAfN" target="_blank">
                  <img src="@/assets/svg/icons/discord_gray.svg" alt="">
                  <p>Discord</p>
                </a>
              </li>
            </ul>
          </div>
          <div class="resources">
            <h4 class="fr">Resources utiles</h4>
            <h4 class="en">Useful resources</h4>
            <ul>
              <li>
                <RouterLink to="">
                  <img src="@/assets/svg/icons/blog.svg" alt="">
                  <p>Blog</p>
                </RouterLink>
              </li>
              <li>
                <RouterLink to="">
                  <img src="@/assets/svg/icons/bug.svg" alt="">
                  <p>Bug Bounty</p>
                </RouterLink>
              </li>
              <li>
                <a href="mailto:kadyteam@gmail.com" target="_blank">
                  <img src="@/assets/svg/icons/mail_gray.svg" alt="">
                  <p class="fr">Contactez nous</p>
                  <p class="en">Contact us</p>
                </a>
              </li>
            </ul>
          </div>
        </div>
      </CategoryItem>
      <CategoryItem title="Modules" id="nav_modules">
        <div class="modules">
          <div class="col">
            <router-link to="/modules/moderation" class="ctg">
              <img src="@/assets/svg/icons/shields.svg" alt="">
              <div class="text">
                <h4>Modération</h4>
                <p>Commandes et outils de modération</p>
              </div>
            </router-link>
            <router-link to="/modules/guild" class="ctg">
              <img src="@/assets/svg/icons/chrono.svg" alt="">
              <div class="text">
                <h4>Gestion du serveur</h4>
                <p>Messages de bienvenue...</p>
              </div>
            </router-link>
            <router-link to="/modules/tickets" class="ctg">
              <img  src="@/assets/svg/icons/ticket.svg" alt="">
              <div class="text">
                <h4>Tickets</h4>
                <p>Système de tickets</p>
              </div>
            </router-link>
          </div>
          <div class="col">
            <router-link to="/modules/tools" class="ctg">
              <img src="@/assets/svg/icons/wrench.svg" alt="">
              <div class="text">
                <h4>Outils</h4>
                <p>Suggestions, avis...</p>
              </div>
            </router-link>
            <router-link to="/modules/fun" class="ctg">
              <img  src="@/assets/svg/icons/ballon.svg" alt="">
              <div class="text">
                <h4>Fun</h4>
                <p>Cookies, jeux...</p>
              </div>
            </router-link>
            <div class="ctg disabled">
              <p style="margin: auto;"><i>En préparation</i></p>
            </div>
          </div>
        </div>
      </CategoryItem>
    </div>

    <div class="right">
      <button class="fr">Se connecter</button>
      <button class="en">Connect</button>
      <LangSelector class_name="navbar_lang_selector"></LangSelector>
    </div>
  </nav>
</template>

<style src="@/assets/css/navbar.css"></style>