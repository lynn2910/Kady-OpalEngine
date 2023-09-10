import { createRouter, createWebHistory } from 'vue-router'
import HomeView from "@/views/RootView.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'root',
      component: HomeView
    }
  ]
})

export default router
