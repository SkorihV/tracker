const DT_RE = /^(\d{2})\.(\d{2})\.(\d{4}) (\d{2}):(\d{2})$/;
const D_RE = /^(\d{2})\.(\d{2})\.(\d{4})$/;

function daysInMonth(y, m) {
  return new Date(y, m, 0).getDate();
}

export function isDateValid(v) {
  if (!v) return true;
  const m = D_RE.exec(v);
  if (!m) return false;
  const d = +m[1];
  const mo = +m[2];
  const y = +m[3];
  return mo >= 1 && mo <= 12 && d >= 1 && d <= daysInMonth(y, mo);
}

export function isDateTimeValid(v) {
  if (!v) return true;
  const m = DT_RE.exec(v);
  if (!m || !isDateValid(m[1] + "." + m[2] + "." + m[3])) return false;
  const h = +m[4];
  const min = +m[5];
  return h <= 23 && min <= 59;
}

/** Правило Vuetify для поля даты «дд.мм.гггг». */
export function dateRule(v) {
  return isDateValid(v) || "Неверная дата (дд.мм.гггг)";
}

/** Правило Vuetify для поля даты со временем «дд.мм.гггг мм:чч». */
export function dateTimeRule(v) {
  return isDateTimeValid(v) || "Неверная дата или время (дд.мм.гггг мм:чч)";
}