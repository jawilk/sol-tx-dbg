<template>
  <table class="registers-table">
    <tr class="header">
      <th class="table-entry">name</th>
      <th class="table-entry2">value</th>
    </tr>
    <tr v-for="register in registers" :key="register.id">
      <td>{{ register.name }}</td>
      <td :class="{ updatedIndices: updatedIndices.includes(register.id) }">
        {{ register.value }}
      </td>
    </tr>
  </table>
</template>

<script>
export default {
  name: "RegistersComp",
  props: ["registers"],
  data() {
    return {
      prevRegisters: [],
      updatedIndices: [],
    };
  },
  methods: {
    flashCells() {
      setTimeout(() => {
        this.updatedIndices = [];
      }, 500);
    },
  },
  mounted() {
    this.prevRegisters = JSON.parse(JSON.stringify(this.registers));
  },
  watch: {
    registers() {
      if (this.prevRegisters.length === 0) {
        this.prevRegisters = JSON.parse(JSON.stringify(this.registers));
        return;
      }
      this.registers.forEach((register, index) => {
        if (register.value !== this.prevRegisters[index].value) {
          this.updatedIndices.push(index);
        }
      });
      if (this.updatedIndices.length > 0) {
        this.flashCells();
      }
      this.prevRegisters = JSON.parse(JSON.stringify(this.registers));
    },
  },
};
</script>

<style>
.registers-table {
  color: #e0e4e6;
  font-size: 13px;
}

.updatedIndices {
  background-color: #595910;
}
</style>