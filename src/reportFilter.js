export function reportFilter(dateFrom, dateTo, filter) {
  return {
    dateFrom,
    dateTo,
    tags: filter.tags,
    client: filter.client,
    user: filter.user,
    search: filter.search,
  };
}