<script setup lang="ts">
import { RouterView } from 'vue-router'
import NavbarComponent from "@/components/navbar/NavbarComponent.vue";
</script>

<template>
  <NavbarComponent></NavbarComponent>
  <div id="router_view">
    <RouterView />
  </div>
</template>
