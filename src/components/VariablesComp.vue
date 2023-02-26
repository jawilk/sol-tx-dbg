<template>
    <table class="variables-table">
      <tr class="header">
        <th class="table-entry">name</th>
        <th class="table-entry2">value</th>
      </tr>
      <tr v-for="(variable, index) in variables" :key="index">
        <td>({{ variable.type }}) {{ variable.name }}</td>
        <td><template v-if="variable.type === variable.value">
           variable not available
        </template>
      <template v-else>
        {{ variable.value }}
      </template>
      </td>
      </tr>
    </table>
  </template>
  
  <script>
  export default {
    name: "VariablesComp",
    props: ["variables"],
  };
  </script>
  
  <style>
  .variables-table {
    color: #e0e4e6;
    font-size: 13px;
  }
  </style>