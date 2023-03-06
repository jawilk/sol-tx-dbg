import { createRouter, createWebHistory } from 'vue-router';
import StartComp from './components/StartComp.vue';
import ProgramChoiceComp from './components/ProgramChoiceComp.vue';
import ProgramComp from './components/ProgramComp.vue';
import NotSupportedComp from './components/NotSupportedComp.vue';


const routes = [
    {
      path: '/',
      name: 'Init',
      component: StartComp
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
    },
    {
      path: '/program/not-supported',
      name: 'notSupported',
      component: NotSupportedComp,
  }
  ];
  
  const router = createRouter({
    history: createWebHistory(),
    routes,
  });
  
export default router;
