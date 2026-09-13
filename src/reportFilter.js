export function today() {
  const d = new Date()
  const mm = String(d.getMonth() + 1).padStart(2, "0")
  const dd = String(d.getDate()).padStart(2, "0")
  return `${d.getFullYear()}-${mm}-${dd}`
}

export function reportFilter(dateFrom, dateTo, filter) {
  return {
    dateFrom,
    dateTo,
    tags: filter.tags,
    client: filter.client,
    user: filter.user,
    search: filter.search,
  }
}