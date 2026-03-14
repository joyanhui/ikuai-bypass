import { loadJson, saveJson } from "./storage.ts";

export type Language = "zh" | "en";

const STORAGE_KEY = "ikb_lang";

type Dict = Record<string, string>;

const ZH: Dict = {
    "app.subtitle": "智能分流管理",
    "tabs.runtime": "运行模式",
    "tabs.config": "配置助手",
    "tabs.cmd": "命令生成器",
    "theme.auto": "主题: 自动",
    "theme.dark": "主题: 深色",
    "theme.light": "主题: 浅色",
    "theme.auto_applied_dark": "主题: 自动(深色)",
    "theme.auto_applied_light": "主题: 自动(浅色)",
    "theme.toggle_hint": "点击切换",
    "lang.toggle": "语言",
    "lang.toggle_hint": "点击切换语言",
    "lang.zh": "中文",
    "lang.en": "English",
    "toast.switch_theme_auto": "已切换到自动主题",
    "toast.switch_theme_dark": "已切换到深色模式",
    "toast.switch_theme_light": "已切换到浅色模式",
    "logs.title": "实时日志",
    "logs.clear": "清空日志",
    "logs.waiting": "等待日志输出...",
    "status.title": "运行状态",
    "status.connecting": "正在连接服务...",
    "status.loading": "加载中...",
    "status.badge.connecting": "连接中",
    "status.badge.running": "运行中",
    "status.cron_running": "定时运行",
    "status.running": "执行中",
    "status.stopped": "已停止",
    "status.standby": "待机",
    "status.sub.running_module": "正在执行模块: {{module}}",
    "status.sub.scheduled_next": "定时模块: {{module}} / 下次执行: {{next}}",
    "status.sub.last_run": "上次执行: {{last}}",
    "status.sub.waiting": "等待启动...",
    "status.sub.disconnected": "无法连接到服务",
    "runtime.action.run_once": "执行一次",
    "runtime.action.stop": "停止执行",
    "runtime.action.stop_cron": "停止定时任务",
    "runtime.action.start_cron": "启动 cron",
    "runtime.action.start_cronaft": "启动 cronAft",
    "runtime.action.clean": "执行清理",
    "runtime.runmode.title": "运行模式",
    "runtime.runmode.once_desc": "执行一次",
    "runtime.runmode.cron_desc": "先执行后定时",
    "runtime.runmode.cronaft_desc": "直接定时",
    "runtime.runmode.clean_desc": "清理模式",
    "runtime.module.title": "分流模块",
    "module.ispdomain": "运营商分流",
    "module.ipgroup": "IPv4分组",
    "module.ipv6group": "IPv6分组",
    "module.ii": "运营商+IPv4",
    "module.ip": "IPv4+IPv6",
    "module.iip": "全模式",
    "cron.label": "Cron",
    "clean.tag": "清理标签",
    "clean.placeholder": "请输入 tag 或 cleanAll",
    "clean.hint": "安全要求：必须显式填写 tag 或 cleanAll。",
    "toast.need_stop_first": "请先停止任务",
    "toast.task_stopped": "任务已停止",
    "toast.clean_requires_tag": "clean 模式必须填写清理标签",
    "toast.clean_running": "正在执行清理...",
    "toast.logs_cleared": "日志已清空",
    "toast.cron_stopped": "定时任务已停止",
    "toast.switch_lang_zh": "已切换到中文",
    "toast.switch_lang_en": "已切换到英文",

    "about.btn": "关于",
    "about.title": "关于",
    "about.subtitle": "项目信息与更新检查",
    "about.section.info": "基本信息",
    "about.section.update": "新版查询",
    "about.section.diagnostics": "一键诊断",
    "about.info.project": "项目",
    "about.info.runtime": "运行环境",
    "about.info.config_path": "配置路径",
    "about.info.http_proxy": "网络设置",
    "about.info.github_proxy": "GitHub 下载加速",
    "about.runtime.tauri": "Tauri App",
    "about.runtime.web": "WebUI",
    "about.update.check": "查询新版",
    "about.update.stable": "正式版",
    "about.update.prerelease": "Pre-release",
    "about.update.open": "打开",
    "about.update.tag": "标签",
    "about.update.time": "时间",
    "about.update.hint.idle": "点击“查询新版”看看有没有更新。",
    "about.update.hint.fetching": "正在查询更新...",
    "about.update.hint.ok": "已获取：{{via}}",
    "about.update.hint.failed": "GitHub API 不通，无法查询",

    "about.diag.generate": "生成报告",
    "about.diag.copy": "复制",
    "about.diag.hint.idle": "生成诊断报告，方便粘贴到 Issue / 讨论区。",
    "about.diag.hint.fetching": "正在生成诊断报告...",
    "about.diag.hint.ok": "已生成（已脱敏）",
    "about.diag.hint.failed": "生成失败",

    "config.title": "配置助手",
    "config.tab.visual": "可视化编辑",
    "config.tab.raw": "文本编辑",
    "config.remote_load": "远程载入",
    "config.save": "保存配置",
    "config.save_keep_comments": "保存配置 (保留注释)",
    "config.basic.title": "基本配置",
    "config.basic.desc":
        "爱快连接、调度、WebUI、分页参数（代理设置在右上角齿轮）",
    "config.basic.toggle": "展开",
    "config.section.ikuai": "iKuai 连接",
    "config.section.schedule": "调度",
    "config.proxy.mode": "连接方式",
    "config.proxy.url": "代理地址",
    "config.proxy.user": "账号（可选）",
    "config.proxy.pass": "密码（可选）",
    "config.proxy.mode.custom": "手动设置",
    "config.proxy.mode.system": "跟随系统",
    "config.proxy.mode.smart": "智能推荐",
    "config.github_proxy": "GitHub 下载加速（可选）",
    "config.github_proxy.smart_hint":
        "只用于下载规则/远程载入，不会影响新版检查。",
    "config.proxy.diff":
        "不确定怎么选？推荐用“智能推荐”。系统里已经设置代理就选“跟随系统”。需要指定代理地址再选“手动设置”。",
    "config.section.webui": "WebUI 设置",
    "config.section.pagination": "数据分页",

    "remote.title": "远程载入",
    "remote.cover_notice": "使用远程配置覆盖本地配置。",
    "remote.proxy_note": "远程载入会按你的网络设置下载。",
    "remote.btn_load": "加载",
    "remote.btn_default": "默认",

    "proxy.btn": "网络设置",
    "proxy.title": "网络设置",
    "proxy.subtitle": "下载规则/检查更新不顺时，再来这里设置",
    "proxy.note": "这里改完后，点“保存配置”才会写入文件并生效。",

    "cron_stop.title": "停止定时任务",
    "cron_stop.subtitle": "停止后，计划任务将不会再按 Cron 自动执行。",
    "common.cancel": "取消",
    "common.close": "关闭",
    "common.confirm": "确认",
    "common.expand": "展开",
    "common.collapse": "收起",
    "cron_stop.confirm": "停止定时任务",
    "cron_stop.notice_title": "注意",
    "cron_stop.status_title": "当前状态",
    "cron_stop.field.module": "模块",
    "cron_stop.field.cron": "Cron",
    "cron_stop.field.next": "下次执行",

    "cmd.title": "命令生成器",
    "cmd.subtitle": "cli 命令构造",
    "cmd.label.run_mode": "运行模式",
    "cmd.label.module": "模块",
    "cmd.label.config_path": "配置路径",
    "cmd.label.export_path": "导出目录",
    "cmd.label.clean_tag": "清理标签",
    "cmd.random_suffix": "随机后缀",
    "cmd.generated": "生成的命令",
    "cmd.copy": "复制",
};

const EN: Dict = {
    "app.subtitle": "Smart Traffic Routing",
    "tabs.runtime": "Runtime",
    "tabs.config": "Config",
    "tabs.cmd": "Command",
    "theme.auto": "Theme: Auto",
    "theme.dark": "Theme: Dark",
    "theme.light": "Theme: Light",
    "theme.auto_applied_dark": "Theme: Auto (Dark)",
    "theme.auto_applied_light": "Theme: Auto (Light)",
    "theme.toggle_hint": "Click to switch",
    "lang.toggle": "Language",
    "lang.toggle_hint": "Click to switch language",
    "lang.zh": "Chinese",
    "lang.en": "English",
    "toast.switch_theme_auto": "Switched to auto theme",
    "toast.switch_theme_dark": "Switched to dark mode",
    "toast.switch_theme_light": "Switched to light mode",
    "logs.title": "Live Logs",
    "logs.clear": "Clear logs",
    "logs.waiting": "Waiting for logs...",
    "status.title": "Status",
    "status.connecting": "Connecting...",
    "status.loading": "Loading...",
    "status.badge.connecting": "Connecting",
    "status.badge.running": "Running",
    "status.cron_running": "Scheduled",
    "status.running": "Running",
    "status.stopped": "Stopped",
    "status.standby": "Standby",
    "status.sub.running_module": "Running module: {{module}}",
    "status.sub.scheduled_next": "Scheduled: {{module}} / Next: {{next}}",
    "status.sub.last_run": "Last run: {{last}}",
    "status.sub.waiting": "Waiting to start...",
    "status.sub.disconnected": "Cannot reach service",
    "runtime.action.run_once": "Run Once",
    "runtime.action.stop": "Stop",
    "runtime.action.stop_cron": "Stop Schedule",
    "runtime.action.start_cron": "Start cron",
    "runtime.action.start_cronaft": "Start cronAft",
    "runtime.action.clean": "Clean",
    "runtime.runmode.title": "Run Mode",
    "runtime.runmode.once_desc": "Run once",
    "runtime.runmode.cron_desc": "Once then schedule",
    "runtime.runmode.cronaft_desc": "Schedule only",
    "runtime.runmode.clean_desc": "Clean mode",
    "runtime.module.title": "Module",
    "module.ispdomain": "ISP Routing",
    "module.ipgroup": "IPv4 Group",
    "module.ipv6group": "IPv6 Group",
    "module.ii": "ISP + IPv4",
    "module.ip": "IPv4 + IPv6",
    "module.iip": "Full Mode",
    "cron.label": "Cron",
    "clean.tag": "Clean Tag",
    "clean.placeholder": "Enter tag or cleanAll",
    "clean.hint": "Safety: you must explicitly provide a tag or cleanAll.",
    "toast.need_stop_first": "Please stop the task first",
    "toast.task_stopped": "Task stopped",
    "toast.clean_requires_tag": "Clean mode requires a tag",
    "toast.clean_running": "Cleaning...",
    "toast.logs_cleared": "Logs cleared",
    "toast.cron_stopped": "Schedule stopped",
    "toast.switch_lang_zh": "Switched to Chinese",
    "toast.switch_lang_en": "Switched to English",

    "about.btn": "About",
    "about.title": "About",
    "about.subtitle": "Project info & update check",
    "about.section.info": "Info",
    "about.section.update": "Updates",
    "about.section.diagnostics": "Diagnostics",
    "about.info.project": "Project",
    "about.info.runtime": "Runtime",
    "about.info.config_path": "Config Path",
    "about.info.http_proxy": "Network",
    "about.info.github_proxy": "GitHub Acceleration",
    "about.runtime.tauri": "Tauri App",
    "about.runtime.web": "WebUI",
    "about.update.check": "Check Updates",
    "about.update.stable": "Stable",
    "about.update.prerelease": "Pre-release",
    "about.update.open": "Open",
    "about.update.tag": "Tag",
    "about.update.time": "Time",
    "about.update.hint.idle":
        "Click “Check Updates” to see if there is a newer version.",
    "about.update.hint.fetching": "Checking for updates...",
    "about.update.hint.ok": "Fetched via: {{via}}",
    "about.update.hint.failed": "GitHub API unreachable, cannot check updates",

    "about.diag.generate": "Generate",
    "about.diag.copy": "Copy",
    "about.diag.hint.idle": "Generate a report for Issue/Discussion.",
    "about.diag.hint.fetching": "Generating diagnostics...",
    "about.diag.hint.ok": "Generated (redacted)",
    "about.diag.hint.failed": "Failed to generate",

    "config.title": "Config Assistant",
    "config.tab.visual": "Visual",
    "config.tab.raw": "YAML",
    "config.remote_load": "Load Remote",
    "config.save": "Save Config",
    "config.save_keep_comments": "Save (Keep Comments)",
    "config.basic.title": "Basic Config",
    "config.basic.desc":
        "iKuai connection, schedule, WebUI, pagination (proxy: top-right gear)",
    "config.basic.toggle": "Expand",
    "config.section.ikuai": "iKuai Connection",
    "config.section.schedule": "Schedule",
    "config.proxy.mode": "Connection Mode",
    "config.proxy.url": "Proxy URL",
    "config.proxy.user": "Username (optional)",
    "config.proxy.pass": "Password (optional)",
    "config.proxy.mode.custom": "Manual",
    "config.proxy.mode.system": "Use System",
    "config.proxy.mode.smart": "Smart (Recommended)",
    "config.github_proxy": "GitHub Download Acceleration (optional)",
    "config.github_proxy.smart_hint":
        "Only for downloading rules/remote template; not used for update check.",
    "config.proxy.diff":
        "Not sure? Use “Smart”. If your system proxy is already set, choose “Use System”. Need to specify a proxy URL? Choose “Manual”.",
    "config.section.webui": "WebUI",
    "config.section.pagination": "Pagination",

    "remote.title": "Remote Load",
    "remote.cover_notice": "Remote config will overwrite local config.",
    "remote.proxy_note":
        "Remote load will download using your network settings.",
    "remote.btn_load": "Load",
    "remote.btn_default": "Default",

    "proxy.btn": "Network",
    "proxy.title": "Network Settings",
    "proxy.subtitle": "Open this only if downloads or update check fail",
    "proxy.note": "Changes are saved only after you click “Save Config”.",

    "cron_stop.title": "Stop Schedule",
    "cron_stop.subtitle":
        "After stopping, cron jobs will no longer run automatically.",
    "common.cancel": "Cancel",
    "common.close": "Close",
    "common.confirm": "Confirm",
    "common.expand": "Expand",
    "common.collapse": "Collapse",
    "cron_stop.confirm": "Stop Schedule",
    "cron_stop.notice_title": "Notice",
    "cron_stop.status_title": "Current Status",
    "cron_stop.field.module": "Module",
    "cron_stop.field.cron": "Cron",
    "cron_stop.field.next": "Next Run",

    "cmd.title": "Command Builder",
    "cmd.subtitle": "The cli core command builder",
    "cmd.label.run_mode": "Run Mode",
    "cmd.label.module": "Module",
    "cmd.label.config_path": "Config Path",
    "cmd.label.export_path": "Export Path",
    "cmd.label.clean_tag": "Clean Tag",
    "cmd.random_suffix": "Random Suffix",
    "cmd.generated": "Generated Command",
    "cmd.copy": "Copy",
};

const DICT: Record<Language, Dict> = { zh: ZH, en: EN };

let currentLanguage: Language = "zh";

export function initLanguage(): Language {
    const saved = loadJson<Language>(STORAGE_KEY, "zh");
    currentLanguage = saved === "en" ? "en" : "zh";
    applyLanguageToDocument(currentLanguage);
    return currentLanguage;
}

export function getLanguage(): Language {
    return currentLanguage;
}

export function setLanguage(lang: Language): void {
    currentLanguage = lang === "en" ? "en" : "zh";
    saveJson(STORAGE_KEY, currentLanguage);
    applyLanguageToDocument(currentLanguage);
}

export function toggleLanguage(): Language {
    const next = currentLanguage === "zh" ? "en" : "zh";
    setLanguage(next);
    return next;
}

export function t(key: string, vars?: Record<string, string | number>): string {
    const raw = DICT[currentLanguage][key] ?? DICT.zh[key] ?? `[${key}]`;
    if (!vars) return raw;
    return Object.keys(vars).reduce((acc, k) => {
        return acc.replaceAll(`{{${k}}}`, String(vars[k]));
    }, raw);
}

export function applyLanguageToDocument(lang: Language): void {
    const html = document.documentElement;
    html.dataset.lang = lang;
    html.lang = lang === "en" ? "en" : "zh-CN";

    const locked = (el: HTMLElement): boolean => {
        return el.dataset.brandLock === "1";
    };

    // text
    document.querySelectorAll<HTMLElement>("[data-i18n]")?.forEach((el) => {
        if (locked(el)) return;
        const key = el.dataset.i18n;
        if (!key) return;
        el.textContent = t(key);
    });

    // placeholder
    document
        .querySelectorAll<HTMLElement>("[data-i18n-placeholder]")
        ?.forEach((el) => {
            if (locked(el)) return;
            const key = el.dataset.i18nPlaceholder;
            if (!key) return;
            (el as HTMLInputElement).placeholder = t(key);
        });

    // title
    document
        .querySelectorAll<HTMLElement>("[data-i18n-title]")
        ?.forEach((el) => {
            if (locked(el)) return;
            const key = el.dataset.i18nTitle;
            if (!key) return;
            el.title = t(key);
        });

    // aria-label
    document
        .querySelectorAll<HTMLElement>("[data-i18n-aria-label]")
        ?.forEach((el) => {
            if (locked(el)) return;
            const key = el.dataset.i18nAriaLabel;
            if (!key) return;
            el.setAttribute("aria-label", t(key));
        });
}
