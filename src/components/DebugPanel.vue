<template>
  <div class="debug-panel">
    <div id="debugDragHandle" title="drag me">~</div>
    <span class="continue" @click="continue_" title="continue" :class="{'deactivated': !isActive}">&#9658;</span>
    <span class="arrow-left-right" @click="next" title="next" :class="{'deactivated': !isActive}">&#8594;</span>
    <!-- <span class="arrow-left-right" @click="next" title="next">&#8592;</span>  -->
    <span class="arrow-down-up" title="step in" :class="{'deactivated': !isActive}">&#8595;</span>
    <span class="arrow-down-up" title="step out" :class="{'deactivated': !isActive}">&#8593;</span>
    <span class="restart" @click="restart" title="restart" :class="{'deactivated': !isActive}">&#8635;</span>
    <span class="stop" @click="stop" title="stop (finish session)" :class="{'deactivated': !isActive}">&#9632;</span>
  </div>
</template>

<script>
export default {
  name: "DebugPanel",
  props: ["isActive"],
  methods: {
    continue_() {
      this.$emit("continue");
    },
    next() {
      if (!this.isActive) return;
      this.$emit("next");
    },
    restart() {
      this.$emit("restart");
    },
    stop() {
      this.$emit("stop");
    },
  },
};
</script>

<style scope>
.deactivated {
  color: #858585 !important;
  cursor: not-allowed !important;
}

.drag-draggable {
  z-index: 15;
}
.debug-panel {
  top: 5px;
  left: 25%;
  position: absolute;
  background-color: transparent;
  width: 200px;
  height: 40px;
  display: flex;
  align-items: center;
  flex-direction: row;
  justify-content: space-evenly;
  z-index: 15;
}

#debugDragHandle {
  font-size: 30px;
  margin-right: 20px;
  color: #61afef;
  text-align: center;
  height: 100%;
  background-color: transparent;
}

.arrow-left-right {
  color: #61afef;
  font-size: 50px;
  margin-bottom: 10px;
  cursor: pointer;
}

.arrow-down-up {
  pointer-events: none;
  color: #858585;
  font-size: 40px;
  cursor: pointer;
  padding: 15px;
  margin-bottom: 10px;
}

.continue {
  height: 100%;
  color: #61afef;
  font-size: 60px;
  margin-bottom: 40px;
  padding-right: 10px;
  padding-left: 7px;
  cursor: pointer;
}

.restart {
  height: 100%;
  color: #98c379;
  font-size: 40px;
  margin-bottom: 20px;
  padding-right: 20px;
  padding-left: 7px;
  cursor: pointer;
}

.stop {
  height: 100%;
  font-size: 40px;
  margin-bottom: 15px;
  padding-right: 20px;
  padding-left: 7px;
  color: #e06c75;
  cursor: pointer;
}
</style>
