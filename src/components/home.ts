/** `/home/theo/x` → `~/x` for display. */
export const home = (p: string) => p.replace(/^\/(home|Users)\/[^/]+/, "~");
