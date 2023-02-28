<template>
  <div class="wrap">
    <p class="program-status">CPI</p>
    <p class="program-name" :title="program_id">{{ program_id }}</p>
  </div>
</template>

<script>
export default {
  name: "NotSupported",
  data() {
    return {
      program_id: "",
    };
  },
  mounted() {
    this.program_id = this.$route.query.program_id;
    window.onload = () => {
      this.showAlert();
    };
  },
  methods: {
    showAlert() {
      alert(
        "program " +
          this.program_id +
          " is not supported\n\nyou can close this window, we will fast forward to after the cpi has finished"
      );
    },
  },
};
</script>