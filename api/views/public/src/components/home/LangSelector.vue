<template>
  <div :class="class_name" class="lang_selector" v-on:click="open_selector(class_name)">
    <img class="flag fr" src="../../assets/svg/flags/french.svg" alt="french_flag_cocorico">
    <img class="flag en" src="../../assets/svg/flags/english.svg" alt="english_flag">
    <h3 class="fr">FR</h3>
    <h3 class="en">EN</h3>
    <img src="../../assets/svg/chevron.svg" alt="selector">
    <div class="lang_selector_hover" :class="class_name">
      <ul>
        <li v-on:click="select_lang(class_name, 'fr')">
          <img src="@/assets/svg/flags/french.svg" alt="french">
          <p>Francais</p>
        </li>
        <li v-on:click="select_lang(class_name, 'en')">
          <img src="@/assets/svg/flags/english.svg" alt="english">
          <p>English</p>
        </li>
      </ul>
    </div>
  </div>
</template>

<script lang="ts">
export default {
  props: {
    class_name: String,
  },
  methods: {
    open_selector(class_name: string | undefined){
      if (!class_name) throw new Error("ID for the lang selector is undefined")

      let selector_element: HTMLElement | null = document.querySelector(`.${class_name}`);
      if (!selector_element) throw new Error(`Cannot find the lang selector div with the id '${class_name}'`);

      let box = selector_element.querySelector(".lang_selector_hover");
      if (!box) throw new Error("Cannot find any element with the class 'lang_selector_hover' in the parent of the lang selector");

      if (box.classList.contains("open")) {
        // close it
        box.classList.remove("open")
      } else {
        // open it :)
        box.classList.add("open")
      }
    },
    select_lang(class_name: string | undefined, lang: string){
      if (!class_name) throw new Error("ID for the lang selector is undefined")

      console.log("lang selected:", lang);

      document.body.setAttribute("lang", lang)
    }
  }
}
</script>