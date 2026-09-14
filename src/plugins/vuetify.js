import "@mdi/font/css/materialdesignicons.css"
import "vuetify/styles"

import { createVuetify } from "vuetify"
import { mdi } from "vuetify/iconsets/mdi"
import { systemIcons } from "../icons/systemIcons"

export default createVuetify({
  icons: {
    defaultSet: "mdi",
    sets: {
      mdi,
      systemIcons,
    },
  },
  defaults: {
    VTextField: { autocomplete: "suppress" },
    VTextarea: { autocomplete: "suppress" },
    VAutocomplete: { autocomplete: "suppress" },
    VCombobox: { autocomplete: "suppress" },
  },
  theme: {
    defaultTheme: "light",
    themes: {
      light: {
        colors: {
          primary: "#4caf50",
        },
      },
      dark: {
        colors: {
          primary: "#4caf50",
        },
      },
    },
  },
})