import "@mdi/font/css/materialdesignicons.css";
import "vuetify/styles";

import { createVuetify } from "vuetify";
import { mdi } from "vuetify/iconsets/mdi";
import { systemIcons } from "../icons/systemIcons";

export default createVuetify({
  icons: {
    defaultSet: "mdi",
    sets: {
      mdi,
      systemIcons,
    },
  },
  theme: {
    themes: {
      light: {
        colors: {
          primary: "#4caf50",
        },
      },
    },
  },
});