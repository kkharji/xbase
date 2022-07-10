export function nameFromPath(path: string): string {
  const pathArray = path.split('/')
  const lastIndex = pathArray.length - 1
  const name = pathArray[lastIndex]
  return name.charAt(0).toUpperCase() + name.slice(1)
}
