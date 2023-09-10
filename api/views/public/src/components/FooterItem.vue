<template>
  <footer>
    <div class="container">
      <div class="yeet">
        <div class="row">
          <img src="/icon_round_little.png" alt="HowAreYou">
          <h1>Kady</h1>
        </div>
        <p class="fr">Le meilleur bot pour les meilleurs serveurs</p>
        <p class="en">The best bot for the best servers</p>
      </div>

      <div class="navigation">
        <div class="list">
          <h2>Modules</h2>
          <ul>
            <li class="fr">
              <RouterLink to="/module/fun">Fun et social</RouterLink>
            </li>
            <li class="fr">
              <RouterLink to="/module/tools">Outils</RouterLink>
            </li>
            <li class="fr">
              <RouterLink to="/module/manage_server">Gestion du serveur</RouterLink>
            </li>
            <li class="en">
              <RouterLink to="/module/fun">Fun & social</RouterLink>
            </li>
            <li class="en">
              <RouterLink to="/module/tools">Tools & Utils</RouterLink>
            </li>
            <li class="en">
              <RouterLink to="/module/manage_server">Server management</RouterLink>
            </li>
          </ul>
        </div>

        <div class="list last">
          <h2>Kady</h2>
          <ul>
            <li class="fr">
              <RouterLink to="/tos#tos">Conditions d’utilisation</RouterLink>
            </li>
            <li class="fr">
              <RouterLink to="/tos#privacy">Politique de confidentialité</RouterLink>
            </li>
            <li class="fr">
              <a href="https://discord.gg/GegJXchAfN" target="_blank">Serveur support</a>
            </li>
            <li class="en">
              <RouterLink to="/tos#tos">Terms and conditions of use</RouterLink>
            </li>
            <li class="en">
              <RouterLink to="/tos#privacy">Privacy policy</RouterLink>
            </li>
            <li class="en">
              <a href="https://discord.gg/GegJXchAfN" target="_blank">Support server</a>
            </li>
          </ul>
        </div>
      </div>

      <div class="contact">
        <h3 class="fr">Nous contacter</h3>
        <h3 class="en">Contact us</h3>
        <div class="row">
          <a href="https://twitter.com/TeamKady" target="_blank">
            <img src="@/assets/svg/icons/twitter.svg" alt="twitter">
          </a>
          <a href="mailto:kadyteam@gmail.com" target="_blank">
            <img src="@/assets/svg/icons/gmail.svg" alt="mail">
          </a>
          <a href="https://discord.gg/GegJXchAfN" target="_blank">
            <img src="@/assets/svg/icons/discord.svg" alt="discord">
          </a>
        </div>
      </div>
    </div>
    <div class="bottom">
      <p>Copyright © 2023 - Kady Team</p>
    </div>
  </footer>
</template>

<style scoped src="@/assets/css/footer.css"></style>
<script setup lang="ts">
</script>