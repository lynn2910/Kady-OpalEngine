<script setup lang="ts">
import FeatureItem from "@/components/home/FeatureItem.vue";
</script>

<template>
  <div class="fr">
    <div class="feature_item">
      <FeatureItem
        title="Gérez votre serveur comme un professionnel"
        :message="`Facilite la gestion de ton serveur en utilisant le système de modération, et utilise les nombreux outils.<br>Avec Kady, vous allez aimer gérer votre serveur !`"
        :more_infos=true
        more_infos_link="/"></FeatureItem>
    </div>
    <div class="feature_item right">
      <FeatureItem
        title="Ayez un profil à votre image"
        :message="`Montrez à tout le monde qui vous êtes et ce que vous aimez !<br>Personnalisez votre profil avec une biographie, des couleurs, et bien plus encore. Obtenez encore plus de choses en utilisant Kady`"
        :more_infos=true
        more_infos_link="/"></FeatureItem>
    </div>
    <div class="feature_item">
      <FeatureItem
        title="Système de niveaux et d’XP"
        :message="`Récompense les membres les plus actifs avec notre système de niveaux.<br>Configure des rôles et salons pour tes membres les plus fidèles !`"
        :more_infos=false
        more_infos_link="/"></FeatureItem>
    </div>
    <div class="feature_item right">
      <FeatureItem
        title="Messages de bienvenue"
        message="Accueillez chaleureusement les nouveaux membres de ton serveur avec de magnifiques messages personnalisés. Informe-les de l’actualité du serveur, et montre leur à quel point ton serveur est incroyable."
        :more_infos=false
        more_infos_link="/"></FeatureItem>
    </div>
  </div>

  <div class="en">
    <div class="feature_item">
      <FeatureItem
        title="Manage your server like a professional"
        :message="`Make it easy to manage your server using the moderation system, and make use of the many tools.<br>With Kady, you'll love managing your server!`"
        :more_infos=true
        more_infos_link="/"></FeatureItem>
    </div>
    <div class="feature_item right">
      <FeatureItem
        title="Create a profile that reflects you"
        :message="`Show everyone who you are and what you love!<br>Customise your profile with a biography, colours, and much more. Get even more out of Kady`"
        :more_infos=true
        more_infos_link="/"></FeatureItem>
    </div>
    <div class="feature_item">
      <FeatureItem
        title="Level and XP system"
        :message="`Reward the most active members with our level system.<br>Configure roles and lounges for your most loyal members!`"
        :more_infos=false
        more_infos_link="/"></FeatureItem>
    </div>
    <div class="feature_item right">
      <FeatureItem
        title="Welcome messages"
        message="Give new members of your server a warm welcome with beautiful personalised messages. Keep them up to date with server news, and show them how incredible your server is."
        :more_infos=false
        more_infos_link="/"></FeatureItem>
    </div>
  </div>









</template>
