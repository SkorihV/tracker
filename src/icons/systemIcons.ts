import {h} from 'vue'
import type {IconProps, IconSet} from 'vuetify'
import iconChart from './main/iconChart.vue'
import iconCheck from './main/iconCheck.vue'
import iconClient from './main/iconClient.vue'
import iconClock from './main/iconClock.vue'
import iconClose from './main/iconClose.vue'
import iconCog from './main/iconCog.vue'
import iconEdit from './main/iconEdit.vue'
import iconExport from './main/iconExport.vue'
import iconFolder from './main/iconFolder.vue'
import iconImport from './main/iconImport.vue'
import iconPause from './main/iconPause.vue'
import iconPlay from './main/iconPlay.vue'
import iconPlus from './main/iconPlus.vue'
import iconSearch from './main/iconSearch.vue'
import iconStop from './main/iconStop.vue'
import iconTimer from './main/iconTimer.vue'
import iconTrash from './main/iconTrash.vue'
import iconUser from './main/iconUser.vue'

//Все новые иконки следует добавлять в эту константу
const mapNameToComponent: any = {
    iconChart,
    iconCheck,
    iconClient,
    iconClock,
    iconClose,
    iconCog,
    iconEdit,
    iconExport,
    iconFolder,
    iconImport,
    iconPause,
    iconPlay,
    iconPlus,
    iconSearch,
    iconStop,
    iconTimer,
    iconTrash,
    iconUser
}

const systemIcons: IconSet = {
    component: (props: IconProps) =>
        h(props.tag, [h(mapNameToComponent[props.icon as string], {class: 'v-icon__svg'})])
}

//Функция для получения списка всех доступных иконок
export const getSystemIconsList = (): string[] => {
    return Object.keys(mapNameToComponent)
}

export {systemIcons}