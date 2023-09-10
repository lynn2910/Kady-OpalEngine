<script setup lang="ts">
defineProps<{
  title: string,
  id: string
}>();

function manipulate_navbar_resources(id: string){
  let element = document.getElementById(id);

  console.log(id, element)

  if (element){
    if (element.classList.contains("opened")) {
      // close it
      element.classList.remove("opened")
    } else {
      element.classList.add("opened")
    }
  }
}

function close_navbar_resource(id: string){
  let element = document.getElementById(id);

  console.log(id, element)

  if (element){
    if (element.classList.contains("opened")) {
      // close it
      element.classList.remove("opened")
    }
  }
}

function open_navbar_resource(id: string){
  let element = document.getElementById(id);

  console.log(id, element)

  if (element){
    if (!element.classList.contains("opened")) {
      // open it
      element.classList.add("opened")
    }
  }
}
</script>

<template>
  <div
    class="nav_item"
    v-on:mouseleave="close_navbar_resource(id)"
    v-on:mouseover="open_navbar_resource(id)">
    <div class="always_visible" @click="manipulate_navbar_resources(id)">
      <h2>{{ title }}</h2>
      <img src="@/assets/svg/chevron.svg" alt="chevron.svg">
    </div>
    <div class="clicked_resources" v-bind:id="id">
      <slot />
    </div>
  </div>
</template>
