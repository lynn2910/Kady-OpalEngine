<script setup lang="ts">
import NewsBox from "@/components/home/newsBox.vue";

defineProps<{
  servers: string
}>()
</script>


<template>
  <div class="icon_plus_sentence">
    <img id="main_icon_svg" src="@/assets/svg/main_icon.svg" alt="HelloThere">
    <h1 class="fr">Le meilleur bot pour les meilleurs serveurs</h1>
    <h1 class="en">The best bot for the best servers</h1>

    <img id="main_line_svg" src="@/assets/svg/main_line.svg" alt="a_line">
  </div>

  <div class="servers">
    <p class="fr">Déjà {{ servers }} serveurs nous font<br>confiance</p>
    <p class="en">Already {{ servers }} servers trust us</p>
  </div>

  <img id="wave" src="@/assets/svg/wave.svg" alt="waving_time">
  <h2 class="fr" id="family_inside_wave">La famille s'agrandit chaque jour !</h2>
  <h2 class="en" id="family_inside_wave">The family is growing every day!</h2>

  <div class="fr">
    <NewsBox
      title="Mise à jour v3.1"
      text="Cette mise à jour apporte de nombreuses nouveautés et changements, ainsi que des optimisations.">
    </NewsBox>
  </div>
  <div class="en">
    <NewsBox
      title="Update v3.1"
      text="This update brings a whole new and improved set of features.">
    </NewsBox>
  </div>
</template>
