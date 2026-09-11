<script setup>
import { state, refreshQuery } from "../store";

function resetFilters() {
  state.filter = { dateFrom: "", dateTo: "", tag: "", client: "", user: "", search: "" };
  refreshQuery();
}

function clearDates() {
  state.filter.dateFrom = "";
  state.filter.dateTo = "";
  refreshQuery();
}
</script>

<template>
  <div class="filterbar">
    <label>С</label>
    <input
      v-model="state.filter.dateFrom"
      placeholder="дд.мм.гггг"
      style="width: 110px"
      class="input"
      @keydown.enter="refreshQuery()"
    />
    <label>По</label>
    <input
      v-model="state.filter.dateTo"
      placeholder="дд.мм.гггг"
      style="width: 110px"
      class="input"
      @keydown.enter="refreshQuery()"
    />

    <select v-model="state.filter.user" @change="refreshQuery()">
      <option value="">Пользователь: (все)</option>
      <option v-for="u in state.users" :key="u" :value="u">{{ u }}</option>
    </select>

    <select v-model="state.filter.tag" @change="refreshQuery()">
      <option value="">Тег: (все)</option>
      <option v-for="t in state.tags" :key="t.id" :value="t.name">{{ t.name }}</option>
    </select>

    <select v-model="state.filter.client" @change="refreshQuery()">
      <option value="">Клиент: (все)</option>
      <option v-for="c in state.clients" :key="c.id" :value="c.name">{{ c.name }}</option>
    </select>

    <button class="small" @click="resetFilters">Сбросить</button>
    <button class="small" @click="clearDates">Очистить даты</button>
  </div>
</template>