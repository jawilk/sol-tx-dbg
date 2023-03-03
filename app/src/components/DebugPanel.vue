<template>
  <div class="debug-panel">
    <div class="debugDragHandle" title="drag me">~</div>
    <span
      class="button-purple"
      @click="continue_"
      title="continue"
      :class="{ deactivated: !isActive }"
      >&#9658;</span
    >
    <span
      class="button-purple"
      @click="next"
      title="next"
      :class="{ deactivated: !isActive }"
      >&#8594;</span
    >
    <span class="deactivated" title="reverse">&#8592;</span>
    <span
      class="button-purple"
      @click="stepIn"
      title="step in"
      :class="{ deactivated: !isActive }"
      >&#8595;</span
    >
    <span
      class="button-purple"
      @click="stepOut"
      title="step out"
      :class="{ deactivated: !isActive }"
      >&#8593;</span
    >
    <span
      class="restart"
      @click="restart"
      title="restart"
      :class="{ deactivated: !isActive && !isRestart }"
      >&#8635;</span
    >
  </div>
</template>

<script>
export default {
  name: "DebugPanel",
  props: {
    isActive: Boolean,
    isRestart: Boolean,
  },
  methods: {
    stepIn() {
      if (!this.isActive) return;
      this.$emit("stepIn");
    },
    stepOut() {
      if (!this.isActive) return;
      this.$emit("stepOut");
    },
    continue_() {
      if (!this.isActive) return;
      this.$emit("continue");
    },
    next() {
      if (!this.isActive) return;
      this.$emit("next");
    },
    restart() {
      if (!this.isActive && !this.isRestart) return;
      window.location.reload();
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
  top: -5px;
  left: 22%;
  gap: 15px;
  position: absolute;
  background-color: transparent;
  width: 300px;
  height: 40px;
  display: flex;
  align-items: center;
  flex-direction: row;
  justify-content: space-evenly;
  z-index: 15;
  font-size: 30px;
  cursor: pointer;
}

.debugDragHandle {
  color: #9945ff;
  text-align: center;
  background-color: transparent;
}

.button-purple {
  color: #9945ff;
  cursor: pointer;
}

.restart {
  color: #14f195;
}
</style>
