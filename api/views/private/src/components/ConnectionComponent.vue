<template>
  <div class="connection">
    <div class="background"></div>
    <div class="conn">
      <div class="box">
        <h2>Connection</h2>

        <div class="informations">
          <div class="info">
            <h2>Identifiant</h2>
            <input id="id" type="text" placeholder="ID" pattern="^[a-zA-Z]+$">
          </div>
          <div class="info">
            <h2>Mot de passe</h2>
            <input id="passwd" type="password" placeholder="Password" pattern="^.+$">
          </div>
        </div>

        <button id="connect_btn" type="button" v-on:click="connect()">Se connecter</button>

        <p class="error_notifier">Des champs sont manquants</p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export default {
  methods: {
    connect(){
      this.close_error();

      let id_raw: HTMLElement | null = document.getElementById("id");
      if (!id_raw) {
        this.new_error("Vous devez préciser un identifiant.");
        return;
      }
      let id = (id_raw as HTMLInputElement).value;
      if (!(/^[a-zA-Z]+$/).test(id)) {
        this.new_error("Cet identifiant est invalide.");
        return;
      }

      let passwd_raw: HTMLElement | null = document.getElementById("passwd");
      if (!passwd_raw) {
        this.new_error("Vous devez donner un mot de passe.");
        return;
      }
      let passwd = (passwd_raw as HTMLInputElement).value;
      if (!(/^.+$/).test(passwd)) {
        this.new_error("Un mot de passe est requis");
        return;
      }

      let btn = document.getElementById("connect_btn");
      if (btn) btn.classList.add("connecting");

      this.$emit("connect_websocket", {id, passwd});
    },
    new_error(error: string){
      let n = document.querySelector(".error_notifier");
      if (n) {
        n.textContent = error;
        n.classList.add("visible");
      }
    },
    close_error(){
      let n = document.querySelector(".error_notifier");
      if (n) n.classList.remove("visible")
    }
  }
}
</script>