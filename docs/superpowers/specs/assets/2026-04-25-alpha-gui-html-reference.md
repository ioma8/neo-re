<!-- About -->
<!DOCTYPE html>

<html class="light" lang="en"><head>
<meta charset="utf-8"/>
<meta content="width=device-width, initial-scale=1.0" name="viewport"/>
<title>AlphaGUI - About</title>
<!-- Material Symbols -->
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<!-- Google Fonts -->
<link href="https://fonts.googleapis.com" rel="preconnect"/>
<link crossorigin="" href="https://fonts.gstatic.com" rel="preconnect"/>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<!-- Tailwind CSS -->
<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<!-- Theme Configuration -->
<script id="tailwind-config">
        tailwind.config = {
            darkMode: "class",
            theme: {
                extend: {
                    "colors": {
                        "secondary-fixed": "#d3e4fe",
                        "surface-container-highest": "#e1e2ee",
                        "on-surface-variant": "#424656",
                        "background": "#faf8ff",
                        "secondary": "#505f76",
                        "on-secondary-fixed": "#0b1c30",
                        "surface-bright": "#faf8ff",
                        "on-primary-fixed": "#001849",
                        "on-tertiary-fixed-variant": "#832600",
                        "outline": "#727687",
                        "inverse-primary": "#b3c5ff",
                        "on-tertiary-container": "#fff6f4",
                        "primary-fixed-dim": "#b3c5ff",
                        "tertiary-container": "#cc4204",
                        "inverse-on-surface": "#eff0fd",
                        "on-error": "#ffffff",
                        "tertiary-fixed": "#ffdbd0",
                        "on-primary": "#ffffff",
                        "surface-container-high": "#e6e7f4",
                        "secondary-fixed-dim": "#b7c8e1",
                        "on-surface": "#191b24",
                        "primary-fixed": "#dae1ff",
                        "surface-tint": "#0054d6",
                        "primary": "#0050cb",
                        "error": "#ba1a1a",
                        "primary-container": "#0066ff",
                        "surface": "#faf8ff",
                        "surface-container-lowest": "#ffffff",
                        "on-tertiary": "#ffffff",
                        "on-primary-fixed-variant": "#003fa4",
                        "on-secondary-container": "#54647a",
                        "inverse-surface": "#2e303a",
                        "on-primary-container": "#f8f7ff",
                        "on-tertiary-fixed": "#390c00",
                        "surface-container-low": "#f2f3ff",
                        "surface-container": "#ecedfa",
                        "on-error-container": "#93000a",
                        "tertiary": "#a33200",
                        "on-secondary": "#ffffff",
                        "tertiary-fixed-dim": "#ffb59d",
                        "outline-variant": "#c2c6d8",
                        "surface-dim": "#d8d9e6",
                        "error-container": "#ffdad6",
                        "secondary-container": "#d0e1fb",
                        "surface-variant": "#e1e2ee",
                        "on-secondary-fixed-variant": "#38485d",
                        "on-background": "#191b24"
                    },
                    "borderRadius": {
                        "DEFAULT": "0.125rem",
                        "lg": "0.25rem",
                        "xl": "0.5rem",
                        "full": "0.75rem"
                    },
                    "spacing": {
                        "margin": "24px",
                        "xs": "4px",
                        "xl": "32px",
                        "gutter": "20px",
                        "sm": "8px",
                        "md": "16px",
                        "lg": "24px"
                    },
                    "fontFamily": {
                        "h3": ["Inter"],
                        "h1": ["Inter"],
                        "mono-sm": ["ui-monospace", "monospace"],
                        "body-md": ["Inter"],
                        "body-lg": ["Inter"],
                        "label-md": ["Inter"],
                        "h2": ["Inter"],
                        "body-sm": ["Inter"]
                    },
                    "fontSize": {
                        "h3": ["20px", { "lineHeight": "28px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "h1": ["30px", { "lineHeight": "38px", "letterSpacing": "-0.02em", "fontWeight": "600" }],
                        "mono-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }],
                        "body-md": ["14px", { "lineHeight": "20px", "fontWeight": "400" }],
                        "body-lg": ["16px", { "lineHeight": "24px", "fontWeight": "400" }],
                        "label-md": ["12px", { "lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "600" }],
                        "h2": ["24px", { "lineHeight": "32px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "body-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }]
                    }
                }
            }
        }
    </script>
<style>
        .material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
        }
    </style>
<style>
    body {
      min-height: max(884px, 100dvh);
    }
  </style>
</head>
<body class="bg-background text-on-background min-h-screen flex flex-col lg:flex-row antialiased">
<!-- NavigationDrawer (Desktop) -->
<aside class="hidden lg:flex flex-col h-full fixed left-0 top-0 z-40 h-screen w-64 border-r border-r border-gray-200 dark:border-gray-800 shadow-none bg-gray-50 dark:bg-gray-950 transition-all duration-200 ease-in-out font-sans text-sm tracking-normal">
<div class="text-xl font-black text-blue-600 dark:text-blue-400 px-6 py-8">AlphaGUI Pro</div>
<nav class="flex flex-col gap-2 px-4 w-full">
<a class="flex items-center gap-3 px-3 py-2 rounded-md text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 w-full transition-colors" href="#">
<span class="material-symbols-outlined">folder_open</span>
<span>Dashboard</span>
</a>
<a class="flex items-center gap-3 px-3 py-2 rounded-md text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 w-full transition-colors" href="#">
<span class="material-symbols-outlined">settings_input_component</span>
<span>SmartApplets</span>
</a>
<a class="flex items-center gap-3 px-3 py-2 rounded-md text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 w-full transition-colors" href="#">
<span class="material-symbols-outlined">developer_board</span>
<span>OS Operations</span>
</a>
<a class="flex items-center gap-3 px-3 py-2 rounded-md bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold border-r-4 border-blue-600 hover:bg-gray-100 dark:hover:bg-gray-800 w-full transition-colors" href="#">
<span class="material-symbols-outlined" style="font-variation-settings: 'FILL' 1;">help_outline</span>
<span>About</span>
</a>
</nav>
</aside>
<div class="flex-1 flex flex-col w-full lg:ml-64">
<!-- TopAppBar (Mobile) -->
<header class="lg:hidden flex items-center justify-between px-4 py-2 w-full sticky top-0 z-50 w-full border-b border-b border-gray-200 dark:border-gray-800 shadow-none bg-white dark:bg-gray-900 font-sans antialiased text-sm font-medium">
<div class="flex items-center gap-2">
<span class="material-symbols-outlined text-blue-600 dark:text-blue-400">settings_input_hdmi</span>
<h1 class="text-lg font-bold tracking-tight text-blue-600 dark:text-blue-400">AlphaGUI</h1>
</div>
<button class="flex items-center justify-center p-1 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors active:opacity-70 transition-opacity rounded-full">
<span class="material-symbols-outlined text-blue-600 dark:text-blue-400">usb</span>
</button>
</header>
<!-- Main Content -->
<main class="flex-1 w-full max-w-[1440px] mx-auto p-md lg:p-margin pb-[80px] lg:pb-margin">
<div class="max-w-3xl mx-auto w-full flex flex-col gap-lg">
<!-- About Card -->
<section class="bg-surface-container-lowest border border-outline-variant rounded-xl p-lg lg:p-margin shadow-[0px_2px_4px_rgba(0,0,0,0.05)] flex flex-col gap-lg">
<div class="flex items-start justify-between border-b border-surface-variant pb-md">
<div class="flex flex-col gap-sm">
<div class="flex items-center gap-md">
<div class="w-12 h-12 rounded-lg bg-surface-container flex items-center justify-center border border-outline-variant">
<span class="material-symbols-outlined text-[28px] text-primary" style="font-variation-settings: 'FILL' 1;">settings_input_hdmi</span>
</div>
<h2 class="font-h1 text-h1 text-on-surface">AlphaGUI</h2>
</div>
<p class="font-body-md text-body-md text-on-surface-variant max-w-xl mt-xs">A comprehensive utility for managing, flashing, and configuring AlphaSmart devices. Built for speed, reliability, and advanced device telemetry retrieval.<br/><br/><strong>Note:</strong> This application has been validated <strong>ONLY</strong> on AlphaSmart Neo. It has <strong>NOT</strong> been validated for AlphaSmart Neo 2 or other AlphaSmart devices.</p>
</div>
<div class="bg-surface-container text-on-surface px-3 py-1 rounded-full font-mono-sm text-mono-sm border border-outline-variant whitespace-nowrap">
                            v2.5.0
                        </div>
</div>
<div class="flex flex-col gap-md">
<h3 class="font-label-md text-label-md text-on-surface uppercase tracking-wider">Resources</h3>
<div class="flex flex-col sm:flex-row gap-md">
<a class="inline-flex items-center justify-center gap-sm bg-surface-container border border-outline-variant hover:bg-surface-container-high text-on-surface px-md py-sm rounded-lg font-label-md text-label-md transition-colors w-full sm:w-auto" href="#">
<span class="material-symbols-outlined text-[18px]">code</span>
                                GitHub Repository
                            </a>
<a class="inline-flex items-center justify-center gap-sm bg-surface-container border border-outline-variant hover:bg-surface-container-high text-on-surface px-md py-sm rounded-lg font-label-md text-label-md transition-colors w-full sm:w-auto" href="#">
<span class="material-symbols-outlined text-[18px]">article</span>
                                Documentation
                            </a>
</div>
</div>
</section>
<!-- Disclaimer Card -->
<section class="bg-error-container border border-error/30 rounded-xl p-md flex items-start gap-md">
<span class="material-symbols-outlined text-error mt-0.5" style="font-variation-settings: 'FILL' 1;">warning</span>
<div class="flex flex-col gap-xs">
<h4 class="font-label-md text-label-md text-on-error-container uppercase tracking-wider">Critical Warning</h4>
<p class="font-body-sm text-body-sm text-on-error-container leading-relaxed">
                            Disclaimer: Use at your own risk. Incorrect use of flashing tools has the potential to brick your device. Ensure you are using verified firmware images and maintaining a stable USB connection during all system operations.
                        </p>
</div>
</section>
</div>
</main>
</div>
<!-- BottomNavBar (Mobile) -->
<nav class="lg:hidden fixed bottom-0 left-0 w-full z-50 flex justify-around items-center px-4 pb-safe h-16 fixed bottom-0 w-full border-t border-t border-gray-200 dark:border-gray-800 shadow-none bg-white dark:bg-gray-900 text-[10px] font-medium uppercase tracking-wider">
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1">dashboard</span>
<span>Dashboard</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1">extension</span>
<span>Applets</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1">memory</span>
<span>System</span>
</a>
<a class="flex flex-col items-center justify-center text-blue-600 dark:text-blue-400 font-bold bg-blue-50/50 dark:bg-blue-900/20 rounded-lg p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1" style="font-variation-settings: 'FILL' 1;">info</span>
<span>About</span>
</a>
</nav>
</body></html>

<!-- OS Operations -->
<!DOCTYPE html>

<html class="light" lang="en"><head>
<meta charset="utf-8"/>
<meta content="width=device-width, initial-scale=1.0" name="viewport"/>
<title>OS Operations - AlphaGUI Manager</title>
<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<script id="tailwind-config">
        tailwind.config = {
            darkMode: "class",
            theme: {
                extend: {
                    "colors": {
                        "secondary-fixed": "#d3e4fe",
                        "surface-container-highest": "#e1e2ee",
                        "on-surface-variant": "#424656",
                        "background": "#faf8ff",
                        "secondary": "#505f76",
                        "on-secondary-fixed": "#0b1c30",
                        "surface-bright": "#faf8ff",
                        "on-primary-fixed": "#001849",
                        "on-tertiary-fixed-variant": "#832600",
                        "outline": "#727687",
                        "inverse-primary": "#b3c5ff",
                        "on-tertiary-container": "#fff6f4",
                        "primary-fixed-dim": "#b3c5ff",
                        "tertiary-container": "#cc4204",
                        "inverse-on-surface": "#eff0fd",
                        "on-error": "#ffffff",
                        "tertiary-fixed": "#ffdbd0",
                        "on-primary": "#ffffff",
                        "surface-container-high": "#e6e7f4",
                        "secondary-fixed-dim": "#b7c8e1",
                        "on-surface": "#191b24",
                        "primary-fixed": "#dae1ff",
                        "surface-tint": "#0054d6",
                        "primary": "#0050cb",
                        "error": "#ba1a1a",
                        "primary-container": "#0066ff",
                        "surface": "#faf8ff",
                        "surface-container-lowest": "#ffffff",
                        "on-tertiary": "#ffffff",
                        "on-primary-fixed-variant": "#003fa4",
                        "on-secondary-container": "#54647a",
                        "inverse-surface": "#2e303a",
                        "on-primary-container": "#f8f7ff",
                        "on-tertiary-fixed": "#390c00",
                        "surface-container-low": "#f2f3ff",
                        "surface-container": "#ecedfa",
                        "on-error-container": "#93000a",
                        "tertiary": "#a33200",
                        "on-secondary": "#ffffff",
                        "tertiary-fixed-dim": "#ffb59d",
                        "outline-variant": "#c2c6d8",
                        "surface-dim": "#d8d9e6",
                        "error-container": "#ffdad6",
                        "secondary-container": "#d0e1fb",
                        "surface-variant": "#e1e2ee",
                        "on-secondary-fixed-variant": "#38485d",
                        "on-background": "#191b24"
                    },
                    "borderRadius": {
                        "DEFAULT": "0.125rem",
                        "lg": "0.25rem",
                        "xl": "0.5rem",
                        "full": "0.75rem"
                    },
                    "spacing": {
                        "margin": "24px",
                        "xs": "4px",
                        "xl": "32px",
                        "gutter": "20px",
                        "sm": "8px",
                        "md": "16px",
                        "lg": "24px"
                    },
                    "fontFamily": {
                        "h3": ["Inter"],
                        "h1": ["Inter"],
                        "mono-sm": ["ui-monospace, monospace"],
                        "body-md": ["Inter"],
                        "body-lg": ["Inter"],
                        "label-md": ["Inter"],
                        "h2": ["Inter"],
                        "body-sm": ["Inter"]
                    },
                    "fontSize": {
                        "h3": ["20px", { "lineHeight": "28px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "h1": ["30px", { "lineHeight": "38px", "letterSpacing": "-0.02em", "fontWeight": "600" }],
                        "mono-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }],
                        "body-md": ["14px", { "lineHeight": "20px", "fontWeight": "400" }],
                        "body-lg": ["16px", { "lineHeight": "24px", "fontWeight": "400" }],
                        "label-md": ["12px", { "lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "600" }],
                        "h2": ["24px", { "lineHeight": "32px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "body-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }]
                    }
                }
            }
        }
    </script>
<style>
        .pb-safe { padding-bottom: env(safe-area-inset-bottom); }
    </style>
<style>
    body {
      min-height: max(884px, 100dvh);
    }
  </style>
</head>
<body class="bg-surface text-on-surface font-body-md text-body-md min-h-screen flex flex-col">
<!-- TopAppBar -->
<header class="flex items-center justify-between px-4 py-2 w-full sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 shadow-none">
<div class="flex items-center gap-md">
<span class="material-symbols-outlined text-blue-600 dark:text-blue-400">settings_input_hdmi</span>
<span class="text-lg font-bold tracking-tight text-blue-600 dark:text-blue-400 font-sans antialiased text-sm font-medium">AlphaGUI</span>
</div>
<div class="flex items-center gap-sm">
<span class="material-symbols-outlined text-blue-600 dark:text-blue-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors p-sm rounded-full active:opacity-70 transition-opacity cursor-pointer">usb</span>
</div>
</header>
<div class="flex flex-1 overflow-hidden">
<!-- NavigationDrawer (Web) -->
<nav class="hidden lg:flex flex-col h-full fixed left-0 top-0 z-40 bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 h-screen w-64 pt-16 font-sans text-sm tracking-normal">
<div class="text-xl font-black text-blue-600 dark:text-blue-400 px-6 py-8">AlphaGUI Pro</div>
<div class="flex flex-col gap-sm px-sm">
<a class="flex items-center gap-md px-md py-sm rounded-DEFAULT text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined">folder_open</span>
<span>Dashboard</span>
</a>
<a class="flex items-center gap-md px-md py-sm rounded-DEFAULT text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined">settings_input_component</span>
<span>SmartApplets</span>
</a>
<a class="flex items-center gap-md px-md py-sm bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold border-r-4 border-blue-600 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined" style="font-variation-settings: 'FILL' 1;">developer_board</span>
<span>OS Operations</span>
</a>
<a class="flex items-center gap-md px-md py-sm rounded-DEFAULT text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined">help_outline</span>
<span>About</span>
</a>
</div>
</nav>
<!-- Main Content -->
<main class="flex-1 lg:ml-64 p-margin lg:p-xl overflow-y-auto pb-24 md:pb-xl">
<div class="max-w-4xl mx-auto space-y-xl">
<!-- Header Area -->
<div>
<h1 class="font-h1 text-h1 text-on-surface mb-xs">OS Operations</h1>
<p class="font-body-lg text-body-lg text-on-surface-variant">Manage device firmware and perform critical system tasks.</p>
</div>
<!-- Primary Action -->
<section class="bg-surface-container-lowest border border-outline-variant rounded-lg p-margin flex flex-col md:flex-row items-start md:items-center justify-between gap-md">
<div>
<h2 class="font-h3 text-h3 text-on-surface flex items-center gap-sm">
<span class="material-symbols-outlined text-primary">backup</span>
                            System Backup
                        </h2>
<p class="font-body-sm text-body-sm text-on-surface-variant mt-sm">Create a complete snapshot of all applets, files, and user settings. Recommended before performing any OS-level operations.</p>
</div>
<button class="bg-primary-container text-on-primary-container font-label-md text-label-md px-lg py-md rounded-DEFAULT flex items-center gap-sm whitespace-nowrap hover:bg-primary-fixed transition-colors shadow-sm">
<span class="material-symbols-outlined text-sm">cloud_download</span>
                        Backup Everything
                    </button>
</section>
<!-- Critical Operations Grid -->
<div class="grid grid-cols-1 md:grid-cols-2 gap-gutter">
<!-- Firmware Reflash -->
<section class="bg-surface-container-lowest border border-error-container rounded-lg overflow-hidden flex flex-col">
<div class="p-md border-b border-surface-variant bg-error-container/20 flex items-center gap-sm">
<span class="material-symbols-outlined text-error">memory</span>
<h3 class="font-h3 text-h3 text-on-surface">Reflash Firmware</h3>
</div>
<div class="p-margin flex-1 flex flex-col justify-between gap-lg">
<p class="font-body-sm text-body-sm text-on-surface-variant">Update or restore the core hardware communication layer. This process takes approximately 3-5 minutes and must not be interrupted.</p>
<div class="bg-surface-container-low p-sm rounded-DEFAULT border border-surface-variant">
<div class="flex justify-between items-center mb-xs">
<span class="font-label-md text-label-md text-on-surface-variant">Current Version</span>
<span class="font-mono-sm text-mono-sm text-on-surface">v2.1.4-beta</span>
</div>
<div class="flex justify-between items-center">
<span class="font-label-md text-label-md text-on-surface-variant">Target Version</span>
<span class="font-mono-sm text-mono-sm text-primary">v2.1.5-stable</span>
</div>
</div>
<button class="w-full bg-error text-on-error font-label-md text-label-md px-md py-sm rounded-DEFAULT hover:opacity-90 transition-opacity flex justify-center items-center gap-sm">
<span class="material-symbols-outlined text-sm">warning</span>
                                Initiate Firmware Flash
                            </button>
</div>
</section>
<!-- System Reflash -->
<section class="bg-surface-container-lowest border border-error-container rounded-lg overflow-hidden flex flex-col">
<div class="p-md border-b border-surface-variant bg-error-container/20 flex items-center gap-sm">
<span class="material-symbols-outlined text-error">developer_board</span>
<h3 class="font-h3 text-h3 text-on-surface">Reflash System</h3>
</div>
<div class="p-margin flex-1 flex flex-col justify-between gap-lg">
<p class="font-body-sm text-body-sm text-on-surface-variant">Reinstall the primary operating system environment. This will erase all non-backed-up data and reset the device to factory defaults.</p>
<div class="bg-surface-container-low p-sm rounded-DEFAULT border border-surface-variant flex items-center gap-md">
<span class="material-symbols-outlined text-tertiary">sd_card_alert</span>
<p class="font-body-sm text-body-sm text-tertiary">Ensure device is connected to stable power source before proceeding.</p>
</div>
<button class="w-full bg-surface-variant text-on-surface-variant font-label-md text-label-md px-md py-sm rounded-DEFAULT hover:bg-surface-dim transition-colors flex justify-center items-center gap-sm border border-outline-variant">
<span class="material-symbols-outlined text-sm">restart_alt</span>
                                Factory Reset &amp; Flash
                            </button>
</div>
</section>
</div>
<!-- Validated SmallROM Operations -->
<section class="bg-surface-container-lowest border border-outline-variant rounded-lg overflow-hidden">
<div class="p-md border-b border-surface-variant flex items-center justify-between">
<h3 class="font-h3 text-h3 text-on-surface flex items-center gap-sm">
<span class="material-symbols-outlined text-secondary">verified</span>
                            Validated SmallROM Operations
                        </h3>
<span class="font-label-md text-label-md bg-secondary-container text-on-secondary-container px-sm py-xs rounded-full">3 Available</span>
</div>
<div class="divide-y divide-surface-variant">
<!-- Op Item -->
<div class="p-md flex items-center justify-between hover:bg-surface-container-low transition-colors group">
<div class="flex items-center gap-md">
<div class="bg-surface-container p-sm rounded-DEFAULT text-on-surface-variant group-hover:text-primary transition-colors">
<span class="material-symbols-outlined">network_check</span>
</div>
<div>
<h4 class="font-label-md text-label-md text-on-surface">Diagnostics Boot Image</h4>
<p class="font-body-sm text-body-sm text-on-surface-variant">Boot into low-level hardware diagnostics mode.</p>
</div>
</div>
<button class="text-primary hover:text-primary-container font-label-md text-label-md px-sm py-xs border border-transparent hover:border-primary-fixed rounded-DEFAULT transition-all">Execute</button>
</div>
<!-- Op Item -->
<div class="p-md flex items-center justify-between hover:bg-surface-container-low transition-colors group">
<div class="flex items-center gap-md">
<div class="bg-surface-container p-sm rounded-DEFAULT text-on-surface-variant group-hover:text-primary transition-colors">
<span class="material-symbols-outlined">data_usage</span>
</div>
<div>
<h4 class="font-label-md text-label-md text-on-surface">NVRAM Clear</h4>
<p class="font-body-sm text-body-sm text-on-surface-variant">Purge non-volatile RAM cache and configuration stubs.</p>
</div>
</div>
<button class="text-primary hover:text-primary-container font-label-md text-label-md px-sm py-xs border border-transparent hover:border-primary-fixed rounded-DEFAULT transition-all">Execute</button>
</div>
<!-- Op Item -->
<div class="p-md flex items-center justify-between hover:bg-surface-container-low transition-colors group">
<div class="flex items-center gap-md">
<div class="bg-surface-container p-sm rounded-DEFAULT text-on-surface-variant group-hover:text-primary transition-colors">
<span class="material-symbols-outlined">usb_off</span>
</div>
<div>
<h4 class="font-label-md text-label-md text-on-surface">Port Reset Routine</h4>
<p class="font-body-sm text-body-sm text-on-surface-variant">Cycle power to I/O controllers safely.</p>
</div>
</div>
<button class="text-primary hover:text-primary-container font-label-md text-label-md px-sm py-xs border border-transparent hover:border-primary-fixed rounded-DEFAULT transition-all">Execute</button>
</div>
</div>
</section>
</div>
</main>
</div>
<!-- BottomNavBar (Mobile) -->
<nav class="md:hidden fixed bottom-0 left-0 w-full z-50 flex justify-around items-center px-4 pb-safe h-16 bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-800 shadow-none text-[10px] font-medium uppercase tracking-wider">
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1">dashboard</span>
<span>Dashboard</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1">extension</span>
<span>Applets</span>
</a>
<a class="flex flex-col items-center justify-center text-blue-600 dark:text-blue-400 font-bold bg-blue-50/50 dark:bg-blue-900/20 rounded-lg p-1 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1" style="font-variation-settings: 'FILL' 1;">memory</span>
<span>System</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform w-16" href="#">
<span class="material-symbols-outlined mb-1">info</span>
<span>About</span>
</a>
</nav>
</body></html>

<!-- SmartApplets -->
<!DOCTYPE html>

<html class="light" lang="en"><head>
<meta charset="utf-8"/>
<meta content="width=device-width, initial-scale=1.0" name="viewport"/>
<title>AlphaGUI - SmartApplets</title>
<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;900&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<script id="tailwind-config">
        tailwind.config = {
            darkMode: "class",
            theme: {
                extend: {
                    "colors": {
                        "secondary-fixed": "#d3e4fe",
                        "surface-container-highest": "#e1e2ee",
                        "on-surface-variant": "#424656",
                        "background": "#faf8ff",
                        "secondary": "#505f76",
                        "on-secondary-fixed": "#0b1c30",
                        "surface-bright": "#faf8ff",
                        "on-primary-fixed": "#001849",
                        "on-tertiary-fixed-variant": "#832600",
                        "outline": "#727687",
                        "inverse-primary": "#b3c5ff",
                        "on-tertiary-container": "#fff6f4",
                        "primary-fixed-dim": "#b3c5ff",
                        "tertiary-container": "#cc4204",
                        "inverse-on-surface": "#eff0fd",
                        "on-error": "#ffffff",
                        "tertiary-fixed": "#ffdbd0",
                        "on-primary": "#ffffff",
                        "surface-container-high": "#e6e7f4",
                        "secondary-fixed-dim": "#b7c8e1",
                        "on-surface": "#191b24",
                        "primary-fixed": "#dae1ff",
                        "surface-tint": "#0054d6",
                        "primary": "#0050cb",
                        "error": "#ba1a1a",
                        "primary-container": "#0066ff",
                        "surface": "#faf8ff",
                        "surface-container-lowest": "#ffffff",
                        "on-tertiary": "#ffffff",
                        "on-primary-fixed-variant": "#003fa4",
                        "on-secondary-container": "#54647a",
                        "inverse-surface": "#2e303a",
                        "on-primary-container": "#f8f7ff",
                        "on-tertiary-fixed": "#390c00",
                        "surface-container-low": "#f2f3ff",
                        "surface-container": "#ecedfa",
                        "on-error-container": "#93000a",
                        "tertiary": "#a33200",
                        "on-secondary": "#ffffff",
                        "tertiary-fixed-dim": "#ffb59d",
                        "outline-variant": "#c2c6d8",
                        "surface-dim": "#d8d9e6",
                        "error-container": "#ffdad6",
                        "secondary-container": "#d0e1fb",
                        "surface-variant": "#e1e2ee",
                        "on-secondary-fixed-variant": "#38485d",
                        "on-background": "#191b24"
                    },
                    "borderRadius": {
                        "DEFAULT": "0.125rem",
                        "lg": "0.25rem",
                        "xl": "0.5rem",
                        "full": "0.75rem"
                    },
                    "spacing": {
                        "margin": "24px",
                        "xs": "4px",
                        "xl": "32px",
                        "gutter": "20px",
                        "sm": "8px",
                        "md": "16px",
                        "lg": "24px"
                    },
                    "fontFamily": {
                        "h3": ["Inter"],
                        "h1": ["Inter"],
                        "mono-sm": ["ui-monospace, monospace"],
                        "body-md": ["Inter"],
                        "body-lg": ["Inter"],
                        "label-md": ["Inter"],
                        "h2": ["Inter"],
                        "body-sm": ["Inter"]
                    },
                    "fontSize": {
                        "h3": ["20px", {"lineHeight": "28px", "letterSpacing": "-0.01em", "fontWeight": "600"}],
                        "h1": ["30px", {"lineHeight": "38px", "letterSpacing": "-0.02em", "fontWeight": "600"}],
                        "mono-sm": ["13px", {"lineHeight": "18px", "fontWeight": "400"}],
                        "body-md": ["14px", {"lineHeight": "20px", "fontWeight": "400"}],
                        "body-lg": ["16px", {"lineHeight": "24px", "fontWeight": "400"}],
                        "label-md": ["12px", {"lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "600"}],
                        "h2": ["24px", {"lineHeight": "32px", "letterSpacing": "-0.01em", "fontWeight": "600"}],
                        "body-sm": ["13px", {"lineHeight": "18px", "fontWeight": "400"}]
                    }
                }
            }
        }
    </script>
<style>
    body {
      min-height: max(884px, 100dvh);
    }
  </style>
</head>
<body class="bg-surface text-on-surface font-body-md min-h-screen flex flex-col md:flex-row pb-24 md:pb-0">
<!-- NavigationDrawer (Desktop) -->
<nav class="hidden lg:flex flex-col h-full fixed left-0 top-0 z-40 bg-gray-50 dark:bg-gray-950 text-blue-600 dark:text-blue-400 font-sans text-sm tracking-normal h-screen w-64 border-r border-r border-gray-200 dark:border-gray-800 shadow-none transition-all duration-200 ease-in-out">
<div class="text-xl font-black text-blue-600 dark:text-blue-400 px-6 py-8">AlphaGUI Pro</div>
<ul class="flex flex-col flex-1 gap-sm px-4">
<li>
<a class="flex items-center gap-md px-4 py-3 rounded-lg text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800" href="#">
<span class="material-symbols-outlined">folder_open</span>
<span>Dashboard</span>
</a>
</li>
<li>
<a class="flex items-center gap-md px-4 py-3 rounded-lg bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold border-r-4 border-blue-600 hover:bg-gray-100 dark:hover:bg-gray-800" href="#">
<span class="material-symbols-outlined">settings_input_component</span>
<span>SmartApplets</span>
</a>
</li>
<li>
<a class="flex items-center gap-md px-4 py-3 rounded-lg text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800" href="#">
<span class="material-symbols-outlined">developer_board</span>
<span>OS Operations</span>
</a>
</li>
<li>
<a class="flex items-center gap-md px-4 py-3 rounded-lg text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800" href="#">
<span class="material-symbols-outlined">help_outline</span>
<span>About</span>
</a>
</li>
</ul>
</nav>
<!-- Main Content Area -->
<div class="flex-1 lg:ml-64 flex flex-col min-h-screen">
<!-- TopAppBar -->
<header class="flex items-center justify-between px-4 py-2 w-full sticky top-0 z-50 bg-white dark:bg-gray-900 text-blue-600 dark:text-blue-400 font-sans antialiased text-sm font-medium w-full border-b border-b border-gray-200 dark:border-gray-800 shadow-none">
<div class="flex items-center gap-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors p-2 rounded-lg active:opacity-70 transition-opacity">
<span class="material-symbols-outlined">settings_input_hdmi</span>
<span class="text-lg font-bold tracking-tight text-blue-600 dark:text-blue-400">AlphaGUI</span>
</div>
<button class="hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors p-2 rounded-lg active:opacity-70 transition-opacity">
<span class="material-symbols-outlined text-gray-500 dark:text-gray-400">usb</span>
</button>
</header>
<!-- Canvas -->
<main class="flex-1 p-margin md:p-xl max-w-[1440px] mx-auto w-full">
<div class="flex items-center justify-between mb-xl">
<div>
<h1 class="font-h1 text-h1 text-on-surface">SmartApplets</h1>
<p class="font-body-lg text-body-lg text-on-surface-variant mt-sm">Manage and install applications on your connected device.</p>
</div>
<div class="hidden md:flex gap-md">
<span class="inline-flex items-center gap-xs px-3 py-1 rounded-full bg-secondary-container/20 text-on-secondary-container font-label-md text-label-md border border-secondary-container">
<span class="w-2 h-2 rounded-full bg-primary"></span>
                        Device Connected
                    </span>
</div>
</div>
<!-- Top Feature Button -->
<div class="mb-xl">
<button class="w-full md:w-auto flex items-center justify-center gap-md bg-primary-container text-on-primary-container font-body-lg text-body-lg py-4 px-6 rounded-xl hover:bg-primary transition-colors shadow-[0_2px_4px_rgba(0,0,0,0.05)] border border-primary-container/20 relative overflow-hidden group">
<div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000"></div>
<span class="material-symbols-outlined" style="font-variation-settings: 'FILL' 1;">cable</span>
<span class="font-semibold">Flash Alpha USB SmartApplet for smartphone connection</span>
</button>
</div>
<!-- Applets List -->
<div class="bg-surface-container-lowest rounded-DEFAULT border border-outline-variant shadow-sm overflow-hidden mb-xl">
<div class="px-lg py-md border-b border-outline-variant bg-surface-container-low/50 flex justify-between items-center">
<h2 class="font-h3 text-h3 text-on-surface">Available Applets</h2>
<span class="font-label-md text-label-md text-on-surface-variant">3 INSTALLED</span>
</div>
<div class="divide-y divide-outline-variant/50">
<!-- Applet Item 1 -->
<label class="flex items-start gap-md p-md hover:bg-surface-container-low transition-colors cursor-pointer group">
<div class="pt-sm">
<input checked="" class="form-checkbox h-5 w-5 text-primary rounded-sm border-outline focus:ring-primary focus:ring-offset-surface-container-lowest bg-surface-container-lowest transition duration-150 ease-in-out" type="checkbox"/>
</div>
<div class="flex-1">
<div class="flex items-center gap-sm">
<span class="font-h3 text-body-lg text-on-surface">AlphaWord</span>
<span class="inline-flex px-2 py-0.5 rounded-full bg-surface-container-highest text-on-surface-variant font-label-md text-[10px]">v3.2</span>
</div>
<p class="font-body-sm text-body-sm text-on-surface-variant mt-xs">Core word processing environment with advanced formatting options.</p>
</div>
<div class="hidden sm:flex flex-col items-end pt-sm">
<span class="font-mono-sm text-mono-sm text-on-surface-variant">124 KB</span>
</div>
</label>
<!-- Applet Item 2 -->
<label class="flex items-start gap-md p-md hover:bg-surface-container-low transition-colors cursor-pointer group">
<div class="pt-sm">
<input checked="" class="form-checkbox h-5 w-5 text-primary rounded-sm border-outline focus:ring-primary focus:ring-offset-surface-container-lowest bg-surface-container-lowest transition duration-150 ease-in-out" type="checkbox"/>
</div>
<div class="flex-1">
<div class="flex items-center gap-sm">
<span class="font-h3 text-body-lg text-on-surface">SpellCheck</span>
<span class="inline-flex px-2 py-0.5 rounded-full bg-surface-container-highest text-on-surface-variant font-label-md text-[10px]">v1.4</span>
</div>
<p class="font-body-sm text-body-sm text-on-surface-variant mt-xs">Comprehensive dictionary and real-time spelling verification engine.</p>
</div>
<div class="hidden sm:flex flex-col items-end pt-sm">
<span class="font-mono-sm text-mono-sm text-on-surface-variant">89 KB</span>
</div>
</label>
<!-- Applet Item 3 -->
<label class="flex items-start gap-md p-md hover:bg-surface-container-low transition-colors cursor-pointer group">
<div class="pt-sm">
<input checked="" class="form-checkbox h-5 w-5 text-primary rounded-sm border-outline focus:ring-primary focus:ring-offset-surface-container-lowest bg-surface-container-lowest transition duration-150 ease-in-out" type="checkbox"/>
</div>
<div class="flex-1">
<div class="flex items-center gap-sm">
<span class="font-h3 text-body-lg text-on-surface">Calculator</span>
<span class="inline-flex px-2 py-0.5 rounded-full bg-surface-container-highest text-on-surface-variant font-label-md text-[10px]">v2.0</span>
</div>
<p class="font-body-sm text-body-sm text-on-surface-variant mt-xs">Standard mathematical operations and basic scientific functions.</p>
</div>
<div class="hidden sm:flex flex-col items-end pt-sm">
<span class="font-mono-sm text-mono-sm text-on-surface-variant">32 KB</span>
</div>
</label>
<!-- Applet Item 4 -->
<label class="flex items-start gap-md p-md hover:bg-surface-container-low transition-colors cursor-pointer group bg-surface-container-lowest">
<div class="pt-sm">
<input class="form-checkbox h-5 w-5 text-primary rounded-sm border-outline focus:ring-primary focus:ring-offset-surface-container-lowest bg-surface-container-lowest transition duration-150 ease-in-out" type="checkbox"/>
</div>
<div class="flex-1">
<div class="flex items-center gap-sm">
<span class="font-h3 text-body-lg text-on-surface">Alpha USB</span>
<span class="inline-flex px-2 py-0.5 rounded-full bg-surface-container-highest text-on-surface-variant font-label-md text-[10px]">v1.1</span>
</div>
<p class="font-body-sm text-body-sm text-on-surface-variant mt-xs">Enables direct file transfer protocol over USB connection.</p>
</div>
<div class="hidden sm:flex flex-col items-end pt-sm">
<span class="font-mono-sm text-mono-sm text-on-surface-variant">45 KB</span>
</div>
</label>
<!-- Applet Item 5 -->
<label class="flex items-start gap-md p-md hover:bg-surface-container-low transition-colors cursor-pointer group bg-surface-container-lowest">
<div class="pt-sm">
<input class="form-checkbox h-5 w-5 text-primary rounded-sm border-outline focus:ring-primary focus:ring-offset-surface-container-lowest bg-surface-container-lowest transition duration-150 ease-in-out" type="checkbox"/>
</div>
<div class="flex-1">
<div class="flex items-center gap-sm">
<span class="font-h3 text-body-lg text-on-surface">QuizMaster</span>
<span class="inline-flex px-2 py-0.5 rounded-full bg-surface-container-highest text-on-surface-variant font-label-md text-[10px]">v1.0</span>
</div>
<p class="font-body-sm text-body-sm text-on-surface-variant mt-xs">Interactive testing and flashcard management system.</p>
</div>
<div class="hidden sm:flex flex-col items-end pt-sm">
<span class="font-mono-sm text-mono-sm text-on-surface-variant">68 KB</span>
</div>
</label>
</div>
</div>
<!-- Action Area -->
<div class="flex flex-col sm:flex-row items-center justify-between gap-md border-t border-outline-variant pt-lg">
<button class="flex items-center gap-sm font-body-sm text-body-sm text-on-surface-variant hover:text-primary transition-colors px-4 py-2 rounded-lg hover:bg-surface-container-low">
<span class="material-symbols-outlined text-[18px]">upload_file</span>
<span>Add new applet from file</span>
</button>
<!-- Assuming changes were made to enable this button visually -->
<button class="w-full sm:w-auto bg-primary text-on-primary font-label-md text-label-md px-6 py-3 rounded-lg hover:bg-primary-container transition-colors shadow-sm focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-surface">
                    FLASH TO DEVICE
                </button>
</div>
</main>
</div>
<!-- BottomNavBar (Mobile) -->
<nav class="md:hidden fixed bottom-0 left-0 w-full z-50 flex justify-around items-center px-4 pb-safe h-16 bg-white dark:bg-gray-900 text-blue-600 dark:text-blue-400 text-[10px] font-medium uppercase tracking-wider fixed bottom-0 w-full border-t border-t border-gray-200 dark:border-gray-800 shadow-none">
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform" href="#">
<span class="material-symbols-outlined">dashboard</span>
<span>Dashboard</span>
</a>
<a class="flex flex-col items-center justify-center text-blue-600 dark:text-blue-400 font-bold bg-blue-50/50 dark:bg-blue-900/20 rounded-lg p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform" href="#">
<span class="material-symbols-outlined" style="font-variation-settings: 'FILL' 1;">extension</span>
<span>Applets</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform" href="#">
<span class="material-symbols-outlined">memory</span>
<span>System</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 active:scale-95 transition-transform" href="#">
<span class="material-symbols-outlined">info</span>
<span>About</span>
</a>
</nav>
</body></html>

<!-- Dashboard -->
<!DOCTYPE html>

<html class="light" lang="en"><head>
<meta charset="utf-8"/>
<meta content="width=device-width, initial-scale=1.0" name="viewport"/>
<title>AlphaGUI Pro - Dashboard</title>
<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<script id="tailwind-config">
        tailwind.config = {
            darkMode: "class",
            theme: {
                extend: {
                    "colors": {
                        "secondary-fixed": "#d3e4fe",
                        "surface-container-highest": "#e1e2ee",
                        "on-surface-variant": "#424656",
                        "background": "#faf8ff",
                        "secondary": "#505f76",
                        "on-secondary-fixed": "#0b1c30",
                        "surface-bright": "#faf8ff",
                        "on-primary-fixed": "#001849",
                        "on-tertiary-fixed-variant": "#832600",
                        "outline": "#727687",
                        "inverse-primary": "#b3c5ff",
                        "on-tertiary-container": "#fff6f4",
                        "primary-fixed-dim": "#b3c5ff",
                        "tertiary-container": "#cc4204",
                        "inverse-on-surface": "#eff0fd",
                        "on-error": "#ffffff",
                        "tertiary-fixed": "#ffdbd0",
                        "on-primary": "#ffffff",
                        "surface-container-high": "#e6e7f4",
                        "secondary-fixed-dim": "#b7c8e1",
                        "on-surface": "#191b24",
                        "primary-fixed": "#dae1ff",
                        "surface-tint": "#0054d6",
                        "primary": "#0050cb",
                        "error": "#ba1a1a",
                        "primary-container": "#0066ff",
                        "surface": "#faf8ff",
                        "surface-container-lowest": "#ffffff",
                        "on-tertiary": "#ffffff",
                        "on-primary-fixed-variant": "#003fa4",
                        "on-secondary-container": "#54647a",
                        "inverse-surface": "#2e303a",
                        "on-primary-container": "#f8f7ff",
                        "on-tertiary-fixed": "#390c00",
                        "surface-container-low": "#f2f3ff",
                        "surface-container": "#ecedfa",
                        "on-error-container": "#93000a",
                        "tertiary": "#a33200",
                        "on-secondary": "#ffffff",
                        "tertiary-fixed-dim": "#ffb59d",
                        "outline-variant": "#c2c6d8",
                        "surface-dim": "#d8d9e6",
                        "error-container": "#ffdad6",
                        "secondary-container": "#d0e1fb",
                        "surface-variant": "#e1e2ee",
                        "on-secondary-fixed-variant": "#38485d",
                        "on-background": "#191b24"
                    },
                    "borderRadius": {
                        "DEFAULT": "0.125rem",
                        "lg": "0.25rem",
                        "xl": "0.5rem",
                        "full": "0.75rem"
                    },
                    "spacing": {
                        "margin": "24px",
                        "xs": "4px",
                        "xl": "32px",
                        "gutter": "20px",
                        "sm": "8px",
                        "md": "16px",
                        "lg": "24px"
                    },
                    "fontFamily": {
                        "h3": ["Inter"],
                        "h1": ["Inter"],
                        "mono-sm": ["ui-monospace, monospace"],
                        "body-md": ["Inter"],
                        "body-lg": ["Inter"],
                        "label-md": ["Inter"],
                        "h2": ["Inter"],
                        "body-sm": ["Inter"]
                    },
                    "fontSize": {
                        "h3": ["20px", { "lineHeight": "28px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "h1": ["30px", { "lineHeight": "38px", "letterSpacing": "-0.02em", "fontWeight": "600" }],
                        "mono-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }],
                        "body-md": ["14px", { "lineHeight": "20px", "fontWeight": "400" }],
                        "body-lg": ["16px", { "lineHeight": "24px", "fontWeight": "400" }],
                        "label-md": ["12px", { "lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "600" }],
                        "h2": ["24px", { "lineHeight": "32px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "body-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }]
                    }
                }
            }
        }
    </script>
<style>
        .material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
        }
    </style>
<style>
    body {
      min-height: max(884px, 100dvh);
    }
  </style>
</head>
<body class="bg-background text-on-background font-body-md text-body-md min-h-screen flex flex-col md:flex-row">
<!-- TopAppBar (Mobile) -->
<header class="flex items-center justify-between px-4 py-2 w-full sticky top-0 z-50 bg-white dark:bg-gray-900 font-sans antialiased text-sm font-medium border-b border-gray-200 dark:border-gray-800 shadow-none md:hidden">
<div class="flex items-center text-blue-600 dark:text-blue-400">
<span class="material-symbols-outlined mr-2">settings_input_hdmi</span>
<span class="text-lg font-bold tracking-tight text-blue-600 dark:text-blue-400">AlphaGUI</span>
</div>
<span class="material-symbols-outlined text-gray-500 dark:text-gray-400">usb</span>
</header>
<!-- NavigationDrawer (Desktop) -->
<nav class="hidden lg:flex flex-col h-full fixed left-0 top-0 z-40 bg-gray-50 dark:bg-gray-950 font-sans text-sm tracking-normal h-screen w-64 border-r border-r border-gray-200 dark:border-gray-800 shadow-none">
<div class="text-xl font-black text-blue-600 dark:text-blue-400 px-6 py-8">AlphaGUI Pro</div>
<div class="flex flex-col space-y-2 mt-4">
<a class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold border-r-4 border-blue-600 flex items-center px-6 py-3 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined mr-4">folder_open</span>
                Dashboard
            </a>
<a class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 flex items-center px-6 py-3 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined mr-4">settings_input_component</span>
                SmartApplets
            </a>
<a class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 flex items-center px-6 py-3 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined mr-4">developer_board</span>
                OS Operations
            </a>
<a class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 flex items-center px-6 py-3 transition-all duration-200 ease-in-out" href="#">
<span class="material-symbols-outlined mr-4">help_outline</span>
                About
            </a>
</div>
</nav>
<!-- Main Content Canvas -->
<main class="flex-1 flex flex-col w-full max-w-[1440px] mx-auto md:ml-64 pb-20 md:pb-0">
<!-- Dashboard Header -->
<div class="px-md lg:px-lg py-lg flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
<div>
<h1 class="font-h1 text-h1 text-on-surface">Dashboard</h1>
<p class="font-body-md text-body-md text-on-surface-variant mt-1">Manage files on connected AlphaSmart device.</p>
</div>
<button class="bg-primary text-on-primary rounded-DEFAULT px-md py-sm font-label-md text-label-md flex items-center gap-2 hover:bg-primary-container transition-colors shadow-sm">
<span class="material-symbols-outlined text-[18px]">cloud_upload</span>
                Backup All Files
            </button>
</div>
<!-- File List Content -->
<div class="px-md lg:px-lg flex-1">
<div class="bg-surface-container-lowest border border-outline-variant rounded-DEFAULT shadow-sm overflow-hidden">
<div class="border-b border-outline-variant bg-surface px-md py-sm flex justify-between items-center">
<span class="font-label-md text-label-md text-on-surface-variant uppercase">Device Files</span>
<span class="font-label-md text-label-md text-on-surface-variant">Connected</span>
</div>
<div class="flex flex-col">
<!-- File Row 1 -->
<div class="flex items-center justify-between p-md border-b border-surface-container-highest hover:bg-surface-container-low transition-colors">
<div class="flex items-center gap-4">
<span class="material-symbols-outlined text-outline">description</span>
<div>
<p class="font-body-md text-body-md text-on-surface font-medium">Chapter_1_Draft.txt</p>
<p class="font-body-sm text-body-sm text-on-surface-variant">14 KB</p>
</div>
</div>
<button class="bg-surface-container-high text-on-surface-variant hover:text-primary rounded-DEFAULT px-sm py-xs font-label-md text-label-md flex items-center gap-1 transition-colors">
<span class="material-symbols-outlined text-[16px]">download</span>
                            Backup
                        </button>
</div>
<!-- File Row 2 -->
<div class="flex items-center justify-between p-md border-b border-surface-container-highest hover:bg-surface-container-low transition-colors">
<div class="flex items-center gap-4">
<span class="material-symbols-outlined text-outline">description</span>
<div>
<p class="font-body-md text-body-md text-on-surface font-medium">Meeting_Notes_Oct.txt</p>
<p class="font-body-sm text-body-sm text-on-surface-variant">4 KB</p>
</div>
</div>
<button class="bg-surface-container-high text-on-surface-variant hover:text-primary rounded-DEFAULT px-sm py-xs font-label-md text-label-md flex items-center gap-1 transition-colors">
<span class="material-symbols-outlined text-[16px]">download</span>
                            Backup
                        </button>
</div>
<!-- File Row 3 -->
<div class="flex items-center justify-between p-md border-b border-surface-container-highest hover:bg-surface-container-low transition-colors">
<div class="flex items-center gap-4">
<span class="material-symbols-outlined text-outline">description</span>
<div>
<p class="font-body-md text-body-md text-on-surface font-medium">Journal_Entry_42.txt</p>
<p class="font-body-sm text-body-sm text-on-surface-variant">28 KB</p>
</div>
</div>
<button class="bg-surface-container-high text-on-surface-variant hover:text-primary rounded-DEFAULT px-sm py-xs font-label-md text-label-md flex items-center gap-1 transition-colors">
<span class="material-symbols-outlined text-[16px]">download</span>
                            Backup
                        </button>
</div>
</div>
</div>
</div>
</main>
<!-- BottomNavBar (Mobile) -->
<nav class="fixed bottom-0 left-0 w-full z-50 flex justify-around items-center px-4 pb-safe h-16 bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-800 shadow-none md:hidden">
<a class="flex flex-col items-center justify-center text-blue-600 dark:text-blue-400 font-bold bg-blue-50/50 dark:bg-blue-900/20 rounded-lg p-1 text-[10px] uppercase tracking-wider active:scale-95 transition-transform w-1/4" href="#">
<span class="material-symbols-outlined mb-1" style="font-variation-settings: 'FILL' 1;">dashboard</span>
<span>Dashboard</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 text-[10px] uppercase tracking-wider active:scale-95 transition-transform w-1/4" href="#">
<span class="material-symbols-outlined mb-1">extension</span>
<span>Applets</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 text-[10px] uppercase tracking-wider active:scale-95 transition-transform w-1/4" href="#">
<span class="material-symbols-outlined mb-1">memory</span>
<span>System</span>
</a>
<a class="flex flex-col items-center justify-center text-gray-500 dark:text-gray-400 p-1 hover:text-blue-500 dark:hover:text-blue-300 text-[10px] uppercase tracking-wider active:scale-95 transition-transform w-1/4" href="#">
<span class="material-symbols-outlined mb-1">info</span>
<span>About</span>
</a>
</nav>
</body></html>

<!-- Connect Device -->
<!DOCTYPE html>

<html lang="en"><head>
<meta charset="utf-8"/>
<meta content="width=device-width, initial-scale=1.0" name="viewport"/>
<title>AlphaGUI - Connection</title>
<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<style>
        .material-symbols-outlined {
            font-family: 'Material Symbols Outlined';
            font-weight: normal;
            font-style: normal;
            font-size: 24px;
            line-height: 1;
            letter-spacing: normal;
            text-transform: none;
            display: inline-block;
            white-space: nowrap;
            word-wrap: normal;
            direction: ltr;
            -webkit-font-feature-settings: 'liga';
            -webkit-font-smoothing: antialiased;
        }
    </style>
<script id="tailwind-config">
        tailwind.config = {
            darkMode: "class",
            theme: {
                extend: {
                    "colors": {
                        "secondary-fixed": "#d3e4fe",
                        "surface-container-highest": "#e1e2ee",
                        "on-surface-variant": "#424656",
                        "background": "#faf8ff",
                        "secondary": "#505f76",
                        "on-secondary-fixed": "#0b1c30",
                        "surface-bright": "#faf8ff",
                        "on-primary-fixed": "#001849",
                        "on-tertiary-fixed-variant": "#832600",
                        "outline": "#727687",
                        "inverse-primary": "#b3c5ff",
                        "on-tertiary-container": "#fff6f4",
                        "primary-fixed-dim": "#b3c5ff",
                        "tertiary-container": "#cc4204",
                        "inverse-on-surface": "#eff0fd",
                        "on-error": "#ffffff",
                        "tertiary-fixed": "#ffdbd0",
                        "on-primary": "#ffffff",
                        "surface-container-high": "#e6e7f4",
                        "secondary-fixed-dim": "#b7c8e1",
                        "on-surface": "#191b24",
                        "primary-fixed": "#dae1ff",
                        "surface-tint": "#0054d6",
                        "primary": "#0050cb",
                        "error": "#ba1a1a",
                        "primary-container": "#0066ff",
                        "surface": "#faf8ff",
                        "surface-container-lowest": "#ffffff",
                        "on-tertiary": "#ffffff",
                        "on-primary-fixed-variant": "#003fa4",
                        "on-secondary-container": "#54647a",
                        "inverse-surface": "#2e303a",
                        "on-primary-container": "#f8f7ff",
                        "on-tertiary-fixed": "#390c00",
                        "surface-container-low": "#f2f3ff",
                        "surface-container": "#ecedfa",
                        "on-error-container": "#93000a",
                        "tertiary": "#a33200",
                        "on-secondary": "#ffffff",
                        "tertiary-fixed-dim": "#ffb59d",
                        "outline-variant": "#c2c6d8",
                        "surface-dim": "#d8d9e6",
                        "error-container": "#ffdad6",
                        "secondary-container": "#d0e1fb",
                        "surface-variant": "#e1e2ee",
                        "on-secondary-fixed-variant": "#38485d",
                        "on-background": "#191b24"
                    },
                    "borderRadius": {
                        "DEFAULT": "0.125rem",
                        "lg": "0.25rem",
                        "xl": "0.5rem",
                        "full": "0.75rem"
                    },
                    "spacing": {
                        "margin": "24px",
                        "xs": "4px",
                        "xl": "32px",
                        "gutter": "20px",
                        "sm": "8px",
                        "md": "16px",
                        "lg": "24px"
                    },
                    "fontFamily": {
                        "h3": ["Inter"],
                        "h1": ["Inter"],
                        "mono-sm": ["ui-monospace, monospace"],
                        "body-md": ["Inter"],
                        "body-lg": ["Inter"],
                        "label-md": ["Inter"],
                        "h2": ["Inter"],
                        "body-sm": ["Inter"]
                    },
                    "fontSize": {
                        "h3": ["20px", { "lineHeight": "28px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "h1": ["30px", { "lineHeight": "38px", "letterSpacing": "-0.02em", "fontWeight": "600" }],
                        "mono-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }],
                        "body-md": ["14px", { "lineHeight": "20px", "fontWeight": "400" }],
                        "body-lg": ["16px", { "lineHeight": "24px", "fontWeight": "400" }],
                        "label-md": ["12px", { "lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "600" }],
                        "h2": ["24px", { "lineHeight": "32px", "letterSpacing": "-0.01em", "fontWeight": "600" }],
                        "body-sm": ["13px", { "lineHeight": "18px", "fontWeight": "400" }]
                    }
                }
            }
        }
    </script>
<style>
    body {
      min-height: max(884px, 100dvh);
    }
  </style>
</head>
<body class="bg-surface text-on-surface min-h-screen flex flex-col font-body-md text-body-md selection:bg-primary-container selection:text-on-primary-container">
<!-- TopAppBar Shared Component -->
<header class="bg-white dark:bg-gray-900 w-full border-b border-gray-200 dark:border-gray-800 flex items-center justify-between px-4 py-2 sticky top-0 z-50">
<div class="flex items-center gap-sm">
<span class="material-symbols-outlined text-blue-600 dark:text-blue-400">settings_input_hdmi</span>
<span class="font-sans antialiased text-sm font-medium text-lg font-bold tracking-tight text-blue-600 dark:text-blue-400">AlphaGUI</span>
</div>
<div class="flex items-center">
<span class="material-symbols-outlined text-blue-600 dark:text-blue-400">usb</span>
</div>
</header>
<!-- Main Canvas -->
<main class="flex-grow flex items-center justify-center p-md md:p-margin">
<!-- Connection Card -->
<div class="bg-surface-container-lowest border border-outline-variant rounded-xl p-xl max-w-xl w-full flex flex-col items-center text-center gap-lg shadow-sm">
<!-- Visual Indicator -->
<div class="relative flex items-center justify-center w-[120px] h-[120px] bg-surface-container rounded-full border-4 border-surface border-dashed">
<span class="material-symbols-outlined text-[64px] text-primary" style="font-variation-settings: 'FILL' 0;">cable</span>
</div>
<!-- Header Text -->
<div class="space-y-sm max-w-md">
<h1 class="font-h2 text-h2 text-on-surface">Awaiting Device</h1>
<p class="font-body-lg text-body-lg text-on-surface-variant">Please connect your AlphaSmart Neo via USB to establish a secure management session.</p>
</div>
<!-- Mobile Instruction Box -->
<div class="bg-secondary-container rounded-lg p-md text-left flex flex-col sm:flex-row items-start gap-md mt-sm border border-outline-variant w-full">
<div class="bg-surface-container-lowest p-sm rounded-full flex-shrink-0">
<span class="material-symbols-outlined text-tertiary" style="font-variation-settings: 'FILL' 1;">info</span>
</div>
<div class="flex flex-col gap-xs pt-xs">
<span class="font-label-md text-label-md text-on-secondary-container uppercase tracking-wider">Mobile Connection</span>
<p class="font-body-sm text-body-sm text-on-secondary-container leading-relaxed">
                        Please ensure you have first installed the <strong>Alpha USB</strong> applet on your AlphaSmart Neo using the desktop version of this app via USB. Run the Alpha USB applet on the device before connecting to this smartphone.
                    </p>
</div>
</div>
<!-- Action Area -->
<div class="w-full pt-sm flex justify-center">
<button class="bg-primary text-on-primary font-label-md text-label-md px-xl py-md rounded-lg hover:bg-primary-container hover:text-on-primary-container transition-colors inline-flex items-center gap-sm uppercase tracking-wide">
<span class="material-symbols-outlined text-[18px]">refresh</span>
                    Scan for Device
                </button>
</div>
</div>
</main>
</body></html>
