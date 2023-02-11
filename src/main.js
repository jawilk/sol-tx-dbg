import { createApp } from 'vue'
import App from './App.vue'

import VueGridLayout from 'vue-grid-layout'
import drag from "v-drag"


createApp(App).use(VueGridLayout).use(drag).mount('#app')
