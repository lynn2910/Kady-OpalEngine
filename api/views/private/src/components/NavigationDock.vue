<template>
  <section class="navigation">
    <div class="head">
      <img src="/icon.png" alt="blep">
      <h2>Kady Security</h2>
    </div>
    <div class="nav">
      <ul>
        <li>
          <img src="@/assets/icons/home.svg" alt="">
          <router-link to="/">Home</router-link>
        </li>
        <li>
          <img src="@/assets/icons/file.svg" alt="">
          <router-link to="/errors">Errors</router-link>
        </li>
        <li>
          <img src="@/assets/icons/db.svg" alt="">
          <router-link to="/db">Database</router-link>
        </li>
        <li>
          <img src="@/assets/icons/honey_comb.svg" alt="">
          <router-link to="/shards">Shards</router-link>
        </li>
        <li>
          <img src="@/assets/icons/layers.svg" alt="">
          <router-link to="/interactions">Interactions</router-link>
        </li>
        <li>
          <img src="@/assets/icons/statistic.svg" alt="">
          <router-link to="/stats">Statistiques</router-link>
        </li>
      </ul>
    </div>
  </section>
</template>

<script lang="ts">
export default {}
</script>