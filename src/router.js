import { createRouter, createWebHistory } from 'vue-router';
import App from './App.vue';
import ProgramChoiceComp from './components/ProgramChoiceComp.vue';
import ProgramComp from './components/ProgramComp.vue';


const routes = [
    {
      path: '/',
      name: 'Init',
      component: App
    },
    {
        path: '/choose',
        name: 'choose',
        component: ProgramChoiceComp,
    },
    {
        path: '/program',
        name: 'program',
        component: ProgramComp,
    }
  ];
  
  const router = createRouter({
    history: createWebHistory(),
    routes,
  });
  
export default router;
