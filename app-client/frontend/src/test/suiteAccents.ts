// Every accent in bato/icons/generate.py ACCENTS, which is canonical — each
// app's brand.ts follows it. Shared by the accent tests and read (by regex) by
// tools/check-contrast.mjs, so keep the one-entry-per-`Name: '#hex'` shape.
export const SUITE_ACCENTS: Record<string, string> = {
  ProximityMusic: '#ef4444', BatoMusic: '#f97316', BatoRemote: '#f59e0b',
  NotEnoughMods: '#84cc16', BatoStore: '#22c55e', BatoMidi: '#10b981',
  BatoFile: '#06b6d4', BatoDisplay: '#3b82f6', BatoHub: '#6366f1',
  BatoHome: '#14b8a6', BatoTemplate: '#8b5cf6', TheHopper: '#d946ef',
  TheWatcher: '#f43f5e', BatoGit: '#f05133', BatoSound: '#0ea5e9',
  BatoFetch: '#a855f7', BatoDeck: '#ec4899', BatoCompose: '#2496ed',
  BatoBrowse: '#eab308', BatoShare: '#2dd4bf', BatoEdit: '#e347c4',
  BatoHealth: '#35c322', BatoScribe: '#14b8a6', BatoMoney: '#21c432',
  BatoAI: '#70d836', BatoGen: '#616a00',
}
