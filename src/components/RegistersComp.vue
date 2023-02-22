<template>
    <table class="registers-table">
        <tr class="header">
          <th class="table-entry">name</th>
          <th class="table-entry2">value</th>
        </tr>
        <tr v-for="register in registers" :key="register.name">
          <td>{{ register.name }}</td>
          <td>{{ register.value }}</td>
        </tr>
      </table>
</template>

<script>
export default {
  name: 'RegistersComp',
  props: ['registers'],
//   watch: {
//     registers() {
//       console.log("HEREEE", this.registers)
//     }
//   },
}

</script>

<style>
.registers-table {
    color: #E0E4E6;
    font-size: 1.2em;
}
</style>