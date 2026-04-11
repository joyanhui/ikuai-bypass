import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

const root = resolve(new URL('..', import.meta.url).pathname)
const metadataPath = resolve(root, 'mobile-version.generated.json')
const metadata = existsSync(metadataPath)
  ? JSON.parse(readFileSync(metadataPath, 'utf8'))
  : null

function patchFileIfExists(filePath, transform) {
  if (!existsSync(filePath)) return
  const original = readFileSync(filePath, 'utf8')
  const updated = transform(original)
  if (updated !== original) {
    writeFileSync(filePath, updated)
  }
}

const androidGradleFiles = [
  resolve(root, 'gen/android/app/build.gradle.kts'),
  resolve(root, 'gen/android/app/build.gradle'),
]

for (const gradleFile of androidGradleFiles) {
  patchFileIfExists(gradleFile, (content) => content
    .replace(/versionName\s*=\s*"[^"]*"/g, metadata?.semverVersion ? `versionName = "${metadata.semverVersion}"` : '$&')
    .replace(/versionName\s+"[^"]*"/g, metadata?.semverVersion ? `versionName "${metadata.semverVersion}"` : '$&')
    .replace(/versionCode\s*=\s*\d+/g, metadata?.versionCode ? `versionCode = ${metadata.versionCode}` : '$&')
    .replace(/versionCode\s+\d+/g, metadata?.versionCode ? `versionCode ${metadata.versionCode}` : '$&'))
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
