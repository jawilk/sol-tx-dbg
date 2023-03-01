<template>
  <span class='plus-btn-wrap'>
  <button class='plus-btn'  @click="addItem"></button>
  </span>
</template>

<script>
export default {
  name: 'AddBtn',
  props: {
  },
  methods: {
    addItem() {
        this.$emit('addItem');
    },
  },
}
</script>

<style scope>

.plus-btn-wrap {
    display: block;
    margin-top: 55px;
    margin-left: 30px;
  -ms-transform: translateY(-50%);
  transform: translateY(-50%);
}

.plus-btn-wrap:hover {
  filter: drop-shadow(0 0 0.75rem rgb(112, 111, 111));
}

.plus-btn {
background-size: cover;
clip-path: polygon(20% 0%, 0% 20%, 30% 50%, 0% 80%, 20% 100%, 50% 70%, 80% 100%, 100% 80%, 70% 50%, 100% 20%, 80% 0%, 50% 30%);
width: 90px;
height: 90px;  
background-image: url('../assets/solana.jpeg');
background-size: 100%;
rotate: 45deg;
border-color:  #2c3e50;
color: #2c3e50;
cursor: pointer; 
}
</style>
