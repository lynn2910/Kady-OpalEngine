<template>
  <NavigationDock></NavigationDock>
  <RouterView/>
  <ConnectionComponent @connect_websocket="connect_websocket"></ConnectionComponent>
</template>

<script lang="ts">
import { RouterView } from 'vue-router'
import NavigationDock from "@/components/NavigationDock.vue";
import ConnectionComponent from "@/components/ConnectionComponent.vue";
import {WebsocketNetwork} from "@/assets/scripts/lib";

export default {
  data(){
    let websocket = new WebsocketNetwork();

    websocket.events.on("connection_refused", () => {
      console.log("Connection refused :(")
      let btn = document.getElementById("connect_btn");
      if (btn) btn.classList.remove("connecting");

      let n = document.querySelector(".error_notifier");
      if (n) {
        n.textContent = "Cannot connect to the API";
        n.classList.add("visible");
      }
    });

    return {
      websocket,
      errors: [],
      database: {
        guilds: {},
        users: {}
      },
      interactions: {
        // contain every type of commands
        commands: {},
        // These 3 can only be enabled or disabled
        buttons: {},
        select_menu: {},
        modals: {}
      },
      shards: {},
      // TODO
      statistics: {}
    }
  },
  methods: {
    connect_websocket({ id, passwd }: { id: string, passwd: string }){
      console.log("connecting...");
      this.websocket.connect(id, passwd, document.location.host)
    }
  },
  components: {
    RouterView,
    NavigationDock,
    ConnectionComponent
  }
}
</script>