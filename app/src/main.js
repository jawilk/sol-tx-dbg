import { createApp } from 'vue'
import App from './App.vue'

import VueGridLayout from 'vue-grid-layout'
import drag from "v-drag"
import router from './router';
import '@fortawesome/fontawesome-free/css/all.css'

createApp(App).use(router).use(VueGridLayout).use(drag).mount('#app')