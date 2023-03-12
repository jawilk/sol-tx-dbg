import { createRouter, createWebHistory } from 'vue-router';
import IntroComp from './components/IntroComp.vue';
import ProgramChoiceComp from './components/ProgramChoiceComp.vue';
import ProgramComp from './components/ProgramComp.vue';
import NotSupportedComp from './components/NotSupportedComp.vue';


const routes = [
    {
      path: '/',
      name: 'Init',
      component: IntroComp
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
