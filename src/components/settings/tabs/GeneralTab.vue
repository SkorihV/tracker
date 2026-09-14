<script setup>
import { ref, computed, onMounted } from "vue"
import { open } from "@tauri-apps/plugin-dialog"
import { useAppStore, DEFAULT_ACCENT_COLOR, DEFAULT_OUR_CAR_COLOR, DEFAULT_TABLE_FONT_FAMILY, DEFAULT_TABLE_FONT_SIZE } from "../../../store.js"

const store = useAppStore()

const color = ref(store.settings.accentColor || DEFAULT_ACCENT_COLOR)
const menu = ref(false)

const ourCarColor = ref(store.settings.ourCarColor || DEFAULT_OUR_CAR_COLOR)
const ourCarMenu = ref(false)

const fontFamily = ref(store.settings.fontFamily || DEFAULT_TABLE_FONT_FAMILY)
const fontSize = ref(store.settings.fontSize || DEFAULT_TABLE_FONT_SIZE)

const fontFamilyOptions = [
  "Roboto",
  "Arial",
  "Verdana",
  "Tahoma",
  "Segoe UI",
  "Times New Roman",
  "Georgia",
  "Courier New",
  "Consolas",
  "monospace",
  "sans-serif",
  "serif",
]

const fontSizeOptions = [10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24]

const swatchStyle = computed(() =>
  ({ backgroundColor: color.value || DEFAULT_ACCENT_COLOR })
)

const ourCarSwatchStyle = computed(() =>
  ({ backgroundColor: ourCarColor.value || DEFAULT_OUR_CAR_COLOR })
)

function apply() {
  store.setAccentColor(color.value)
  menu.value = false
}

function applyFromInput() {
  if (/^#[0-9a-fA-F]{6}$/.test(color.value.trim())) {
    store.setAccentColor(color.value.trim())
  }
}

function resetDefault() {
  color.value = DEFAULT_ACCENT_COLOR
  store.setAccentColor(DEFAULT_ACCENT_COLOR)
}

function applyOurCar() {
  store.setOurCarColor(ourCarColor.value)
  ourCarMenu.value = false
}

function applyOurCarFromInput() {
  if (/^#[0-9a-fA-F]{6}$/.test(ourCarColor.value.trim())) {
    store.setOurCarColor(ourCarColor.value.trim())
  }
}

function resetOurCarDefault() {
  ourCarColor.value = DEFAULT_OUR_CAR_COLOR
  store.setOurCarColor(DEFAULT_OUR_CAR_COLOR)
}

function applyFont() {
  store.setTableFont(fontFamily.value, fontSize.value)
}

function resetFontDefault() {
  fontFamily.value = DEFAULT_TABLE_FONT_FAMILY
  fontSize.value = DEFAULT_TABLE_FONT_SIZE
  store.setTableFont(DEFAULT_TABLE_FONT_FAMILY, DEFAULT_TABLE_FONT_SIZE)
}

const defaultBackupsPath = ref("")
const backupsPath = ref("")

onMounted(async () => {
  defaultBackupsPath.value = await store.getDefaultBackupsDir()
  const effective = await store.effectiveBackupsDir()
  backupsPath.value = effective
})

async function pickBackupsDir() {
  const picked = await open({ directory: true, multiple: false })
  if (picked) {
    const p = String(picked)
    await store.setBackupsDir(p)
    backupsPath.value = p
  }
}

async function resetBackupsDir() {
  await store.setBackupsDir(defaultBackupsPath.value)
  backupsPath.value = defaultBackupsPath.value
}
</script>

<template>
  <div class="d-flex flex-column ga-4">
    <div class="d-flex align-center ga-3 flex-wrap">
      <span class="text-body-2">Цвет кнопок и заголовков:</span>
      <v-menu v-model="menu" :close-on-content-click="false" location="bottom start">
        <template #activator="{ props: menuProps }">
          <v-btn
            v-bind="menuProps"
            icon
            size="small"
            variant="flat"
            aria-label="Цвет кнопок"
            :style="swatchStyle"
          />
        </template>
        <v-sheet class="pa-2 d-flex flex-column ga-2" border>
          <v-color-picker v-model="color" mode="hex" hide-inputs :width="280" />
          <v-btn color="primary" variant="flat" density="compact" @click="apply">
            Применить
          </v-btn>
        </v-sheet>
      </v-menu>

      <v-text-field
        v-model="color"
        label="Цвет (hex)"
        placeholder="#4caf50"
        density="compact"
        variant="outlined"
        hide-details
        class="mono"
        style="max-width: 160px"
        @keydown.enter="applyFromInput"
        @blur="applyFromInput"
        @update:model-value="color = $event.trim()"
      />

      <v-btn variant="tonal" @click="resetDefault">
        По умолчанию
      </v-btn>
    </div>

    <div class="d-flex align-center ga-3 flex-wrap">
      <span class="text-body-2">Цвет ячейки «Наша машина»:</span>
      <v-menu v-model="ourCarMenu" :close-on-content-click="false" location="bottom start">
        <template #activator="{ props: menuProps }">
          <v-btn
            v-bind="menuProps"
            icon
            size="small"
            variant="flat"
            aria-label="Цвет ячейки Наша машина"
            :style="ourCarSwatchStyle"
          />
        </template>
        <v-sheet class="pa-2 d-flex flex-column ga-2" border>
          <v-color-picker v-model="ourCarColor" mode="hex" hide-inputs :width="280" />
          <v-btn color="primary" variant="flat" density="compact" @click="applyOurCar">
            Применить
          </v-btn>
        </v-sheet>
      </v-menu>

      <v-text-field
        v-model="ourCarColor"
        label="Цвет (hex)"
        placeholder="#2196f3"
        density="compact"
        variant="outlined"
        hide-details
        class="mono"
        style="max-width: 160px"
        @keydown.enter="applyOurCarFromInput"
        @blur="applyOurCarFromInput"
        @update:model-value="ourCarColor = $event.trim()"
      />

      <v-btn variant="tonal" @click="resetOurCarDefault">
        По умолчанию
      </v-btn>
    </div>

    <v-divider />

    <div class="d-flex flex-column ga-3">
      <span class="text-body-2">Шрифт таблицы задач:</span>
      <div class="d-flex align-center ga-3 flex-wrap">
        <v-select
          v-model="fontFamily"
          :items="fontFamilyOptions"
          label="Семейство"
          density="compact"
          variant="outlined"
          hide-details
          style="max-width: 200px"
          @update:model-value="applyFont"
        />
        <v-select
          v-model="fontSize"
          :items="fontSizeOptions"
          label="Размер (px)"
          density="compact"
          variant="outlined"
          hide-details
          style="max-width: 130px"
          @update:model-value="applyFont"
        />
        <v-btn variant="tonal" @click="resetFontDefault">
          По умолчанию
        </v-btn>
      </div>
      <span class="text-caption text-medium-emphasis" :style="{ fontFamily: fontFamily, fontSize: fontSize + 'px' }">
        Пример: АаБбь 0Oo 12345
      </span>
    </div>

    <v-divider />

    <div class="d-flex align-center ga-3 flex-wrap">
      <span class="text-body-2">Тема оформления:</span>
      <v-switch
        :model-value="store.settings.theme === 'dark'"
        label="Тёмная"
        density="compact"
        hide-details
        color="primary"
        @update:model-value="store.setTheme($event ? 'dark' : 'light')"
      />
      <v-btn
        variant="tonal"
        :disabled="store.settings.theme === 'light'"
        @click="store.setTheme('light')"
      >
        Светлая
      </v-btn>
    </div>

    <v-divider />

    <div class="d-flex flex-column ga-3">
      <span class="text-body-2">Папка бэкапов:</span>
      <div class="d-flex align-center ga-3 flex-wrap">
        <span
          class="mono text-caption text-medium-emphasis flex-grow-1"
          style="min-width: 160px; max-width: 420px; overflow-wrap: anywhere"
        >
          {{ backupsPath || "…" }}
        </span>
        <v-btn variant="tonal" prepend-icon="systemIcons:iconFolder" @click="pickBackupsDir">
          Выбрать
        </v-btn>
        <v-btn
          variant="tonal"
          :disabled="!store.settings.backupDir"
          @click="resetBackupsDir"
        >
          По умолчанию
        </v-btn>
      </div>
    </div>
  </div>
</template>