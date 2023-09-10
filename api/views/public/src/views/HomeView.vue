<script setup lang="ts">
import TheWelcome from '../components/WelcomeMain.vue'
import FeatureView from "@/components/home/FeatureView.vue";
import FooterItem from "@/components/FooterItem.vue";
</script>

<template>
  <header>
    <TheWelcome servers="116" />
  </header>
  <main>
    <FeatureView></FeatureView>
  </main>
  <FooterItem></FooterItem>
</template>

<style scoped>
body {
  margin-top: 100px;
}
</style>