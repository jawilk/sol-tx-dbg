<template>
  <div>
  <button class="btn-start" @click="next"/>
  <button class="btn-stop" @click="next"/>
  </div>
</template>

<script>
export default {
  name: 'DebugPanel',
  methods: {
    next() {
      console.log("BUTTON NEXT")
      this.$emit('next');
    }
  }
}
</script>

<style scope>

.btn-start {
  cursor: pointer;
  width: 25px;
  height: 25px;
  box-sizing: border-box;
  border-style: solid;
  border-width: 12px 0px 12px 30px;
  background: transparent;
  border-color: transparent transparent transparent #347c1d;
}

.btn-stop {
  float: right;
  cursor: pointer;
  width: 25px;
  height: 25px;
  border-style: solid;
  border-width: 3px;
  background: transparent;
  border-color: #c52525;
}

</style>
