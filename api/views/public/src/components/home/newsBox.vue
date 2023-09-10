<script lang="ts" setup>
defineProps<{
  title: string,
  text: string,
}>()
</script>

<template>
  <div class="news_box">
    <h1>{{ title }}</h1>
    <p>{{ text }}</p>
  </div>
</template>