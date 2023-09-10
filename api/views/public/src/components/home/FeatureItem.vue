<script setup lang="ts">
defineProps<{
  title: string,
  message: string,
  more_infos: boolean,
  more_infos_link: string | null
}>()
</script>

<template>
  <img src="/blep.png" alt="blp">

  <div class="feature_item_content">
    <h2>{{ title }}</h2>
    <p v-html="message"></p>
    <div class="buttons">
      <button class="add_discord fr">Ajouter à Discord</button>
      <button class="add_discord en">Add to Discord</button>
      <a v-if="more_infos" href="{{ more_infos_link }}" target="_blank">
        <button class="learn_more fr">En savoir plus</button>
        <button class="learn_more en">More information</button>
      </a>
    </div>
  </div>
</template>