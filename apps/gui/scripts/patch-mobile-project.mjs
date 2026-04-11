import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const metadataPath = resolve(root, 'mobile-version.generated.json')
const metadata = existsSync(metadataPath)
  ? JSON.parse(readFileSync(metadataPath, 'utf8'))
  : null

function upsertJavaProperties(content, entries) {
  const lines = content ? content.split(/\r?\n/) : []
  const out = []
  const pending = new Map(Object.entries(entries).filter(([, value]) => value != null))

  for (const line of lines) {
    const match = line.match(/^\s*([^#!][^=:#\s]*)\s*[=:].*$/)
    if (match && pending.has(match[1])) {
      out.push(`${match[1]}=${pending.get(match[1])}`)
      pending.delete(match[1])
    } else if (line.length > 0 || out.length > 0) {
      out.push(line)
    }
  }

  for (const [key, value] of pending) {
    out.push(`${key}=${value}`)
  }

  return out.join('\n').replace(/\n*$/, '\n')
}

function patchFileIfExists(filePath, transform) {
  if (!existsSync(filePath)) return
  const original = readFileSync(filePath, 'utf8')
  const updated = transform(original)
  if (updated !== original) {
    writeFileSync(filePath, updated)
  }
}

const androidAppRoot = resolve(root, 'gen/android/app')
if (existsSync(androidAppRoot)) {
  const androidGradleFiles = [
    resolve(androidAppRoot, 'build.gradle.kts'),
    resolve(androidAppRoot, 'build.gradle'),
  ]

  for (const gradleFile of androidGradleFiles) {
    patchFileIfExists(gradleFile, (content) => content
      .replace(/versionName\s*=\s*"[^"]*"/g, metadata?.semverVersion ? `versionName = "${metadata.semverVersion}"` : '$&')
      .replace(/versionName\s+"[^"]*"/g, metadata?.semverVersion ? `versionName "${metadata.semverVersion}"` : '$&')
      .replace(/versionCode\s*=\s*\d+/g, metadata?.versionCode ? `versionCode = ${metadata.versionCode}` : '$&')
      .replace(/versionCode\s+\d+/g, metadata?.versionCode ? `versionCode ${metadata.versionCode}` : '$&'))
  }

  const androidTauriProperties = resolve(androidAppRoot, 'tauri.properties')
  if (metadata?.semverVersion || metadata?.versionCode) {
    const original = existsSync(androidTauriProperties)
      ? readFileSync(androidTauriProperties, 'utf8')
      : ''
    const updated = upsertJavaProperties(original, {
      'tauri.android.versionName': metadata?.semverVersion,
      'tauri.android.versionCode': metadata?.versionCode,
    })
    if (updated !== original) {
      writeFileSync(androidTauriProperties, updated)
    }
  }
}

const androidIconSource = resolve(root, 'icons/android')
const androidResTarget = resolve(root, 'gen/android/app/src/main/res')
if (existsSync(androidIconSource) && existsSync(androidResTarget)) {
  cpSync(androidIconSource, androidResTarget, { recursive: true, force: true })
}

const iosIconSource = resolve(root, 'icons/ios')
const appleGenRoot = resolve(root, 'gen/apple')
if (existsSync(iosIconSource) && existsSync(appleGenRoot)) {
  for (const entry of readdirSync(appleGenRoot)) {
    const entryPath = resolve(appleGenRoot, entry)
    if (!statSync(entryPath).isDirectory()) continue
    const appIconSet = resolve(entryPath, 'Assets.xcassets/AppIcon.appiconset')
    if (existsSync(appIconSet) && statSync(appIconSet).isDirectory()) {
      mkdirSync(appIconSet, { recursive: true })
      cpSync(iosIconSource, appIconSet, { recursive: true, force: true })
    }
  }
}
