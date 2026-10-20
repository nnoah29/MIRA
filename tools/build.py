#!/usr/bin/env python3
"""Génère les pages statiques (fr/ et en/) à partir des textes ci-dessous."""
import html, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else None

ICONS = {
    "download": "M12 3v12M7 10l5 5 5-5M4 20h16",
    "code": "M8 7l-5 5 5 5M16 7l5 5-5 5",
    "copy": "M8 8h12v12H8zM4 16V4h12",
    "check": "M5 12l5 5 9-10",
}
def icon(n):
    return f'<span class="icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" aria-hidden="true"><path d="{ICONS[n]}"/></svg></span>'

RELEASE = "https://github.com/nnoah29/MIRA/releases/download/v1.0.1/"
FILES = [
    ("mira-mail_1.0.1_amd64.deb", "sudo apt install ./mira-mail_1.0.1_amd64.deb"),
    ("mira-mail-1.0.1-1.x86_64.rpm", "sudo dnf install ./mira-mail-1.0.1-1.x86_64.rpm"),
    ("MIRA-1.0.1-x86_64.AppImage", "chmod +x MIRA-1.0.1-x86_64.AppImage && ./MIRA-1.0.1-x86_64.AppImage"),
    ("mira-mail-1.0.1-x86_64.tar.gz", "tar xzf mira-mail-1.0.1-x86_64.tar.gz"),
    ("mira-mail-1.0.1-android-arm64-v8a.apk", None),
]
INSTALL = "curl -fsSL https://github.com/nnoah29/MIRA/releases/latest/download/install.sh | bash"
LOOK = [("GM", "#66a1cf"), ("OU", "#83bb85"), ("YA", "#cbaa66"), ("IM", "#d1839f")]

FR = dict(
    lang="fr", other="en", switch="EN", switch_label="View this page in English",
    home="Accueil", skip="Aller au contenu",
    t_home="MIRA, un client mail pour Linux et Android", t_privacy="Confidentialité · MIRA", t_terms="Conditions d'utilisation · MIRA",
    desc="MIRA est un client mail pour Linux et Android, libre et gratuit. Un client mail comme un autre, en plus joli et plus amusant à utiliser.",
    nav=("Atouts", "Comptes", "Confidentialité", "Télécharger"),
    footer=("MIRA · Noah Toffa · Licence MIT", "Confidentialité", "Conditions", "Code source", "Contact"),
    h1=("Juste un client mail,", "en plus joli"),
    lede="MIRA lit et envoie vos mails, sur Linux et sur Android. Je l'ai faite parce que les clients mail que j'ai trouvés sous Linux ne me plaisaient pas : trop austères, payants, ou coincés dans un navigateur.",
    dl="Télécharger MIRA", src="Code source", tags="Linux · Android · Gratuit · Licence MIT",
    shot_alt="La fenêtre de MIRA : dossiers à gauche, boîte de réception au centre, lettre d'information ouverte à droite.",
    f_title="Ce qu'il y a dedans",
    strengths=[
        ("Look", "Un peu de caractère", "Grands titres, angles droits, une seule couleur vive. Thème sombre ou clair, en français ou en anglais."),
        ("Couleurs", "La vôtre, pas la mienne", "Choisissez la couleur d'accent, ou laissez MIRA prendre celles de votre fond d'écran."),
        ("Comptes", "Tous au même endroit", "Gmail, Outlook, Yahoo ou n'importe quel serveur IMAP, avec une réception unifiée si vous le voulez."),
        ("Hors ligne", "Ça marche sans réseau", "Vos mails sont copiés sur votre appareil. Lisez et répondez dans le train : tout part au retour du réseau."),
        ("Au quotidien", "Les petits plus", "Recherche dans tout le texte, reporter un mail à plus tard, annuler un envoi, programmer un message, raccourcis clavier."),
        ("Vie privée", "Rien ne passe par chez moi", "Aucun serveur MIRA. Les images distantes et les traceurs sont bloqués tant que vous ne les autorisez pas."),
    ],
    a_title="Vos comptes",
    accounts=[("Gmail", "Connexion Google"), ("Outlook, Microsoft 365", "Connexion Microsoft"), ("Yahoo", "Mot de passe d'application"), ("Tout compte IMAP", "Serveur et mot de passe")],
    d_title="Télécharger",
    systems=[("Debian, Ubuntu", "Debian, Ubuntu, Mint, Pop!_OS"), ("Fedora", "Fedora, RHEL, Rocky, Alma, openSUSE"), ("AppImage", "Toute distribution Linux"), ("Arch", "Arch, Manjaro, EndeavourOS"), ("Android", "Téléphones Android 7 et suivants")],
    get="Télécharger", then="Puis, dans un terminal", copy="Copier", copied="Copié",
    install_label="Installer et mettre à jour · Linux, sans sudo", install_note="Relancez la même commande pour passer à la dernière version.",
    system_label="Système",
    req=("Linux : glibc 2.34 ou plus récente (Ubuntu 22.04, Debian 12, Fedora 35 et suivantes) et GTK 3. Pour ranger les mots de passe, un trousseau comme GNOME Keyring ou KWallet. Sommes de contrôle publiées avec chaque ", "version", "."),
    docs_label="Documents", d_privacy="Confidentialité", d_terms="Conditions d'utilisation", branch=0,
)
EN = dict(
    lang="en", other="fr", switch="FR", switch_label="Voir cette page en français",
    home="Home", skip="Skip to content",
    t_home="MIRA, a mail client for Linux and Android", t_privacy="Privacy · MIRA", t_terms="Terms of use · MIRA",
    desc="MIRA is a free, open-source mail client for Linux and Android. A mail client like any other, just prettier and more fun to use.",
    nav=("Features", "Accounts", "Privacy", "Download"),
    footer=("MIRA · Noah Toffa · MIT licence", "Privacy", "Terms", "Source code", "Contact"),
    h1=("Just a mail client,", "but prettier"),
    lede="MIRA reads and sends your mail, on Linux and on Android. I made it because the mail clients I found on Linux didn't appeal to me: too austere, paid, or stuck in a browser.",
    dl="Download MIRA", src="Source code", tags="Linux · Android · Free · MIT licence",
    shot_alt="The MIRA window: folders on the left, inbox in the middle, a newsletter open on the right.",
    f_title="What's inside",
    strengths=[
        ("Look", "A bit of character", "Big headlines, square corners, one bright colour. Dark or light theme, in English or French."),
        ("Colours", "Yours, not mine", "Pick the accent colour, or let MIRA take the colours of your wallpaper."),
        ("Accounts", "All in one place", "Gmail, Outlook, Yahoo or any IMAP server, with a unified inbox if you want one."),
        ("Offline", "Works without a network", "Your mail is copied to your device. Read and reply on the train: everything goes out when the network returns."),
        ("Day to day", "Small extras", "Full-text search, snooze a message, undo send, schedule a message, keyboard shortcuts."),
        ("Privacy", "Nothing goes through me", "No MIRA server. Remote images and trackers stay blocked until you allow them."),
    ],
    a_title="Your accounts",
    accounts=[("Gmail", "Google sign-in"), ("Outlook, Microsoft 365", "Microsoft sign-in"), ("Yahoo", "App password"), ("Any IMAP account", "Server and password")],
    d_title="Download",
    systems=[("Debian, Ubuntu", "Debian, Ubuntu, Mint, Pop!_OS"), ("Fedora", "Fedora, RHEL, Rocky, Alma, openSUSE"), ("AppImage", "Any Linux distribution"), ("Arch", "Arch, Manjaro, EndeavourOS"), ("Android", "Phones running Android 7 and later")],
    get="Download", then="Then, in a terminal", copy="Copy", copied="Copied",
    install_label="Install and update · Linux, no sudo", install_note="Run the same command again to get the latest version.",
    system_label="System",
    req=("Linux: glibc 2.34 or newer (Ubuntu 22.04, Debian 12, Fedora 35 and later) and GTK 3. To store passwords, a keyring such as GNOME Keyring or KWallet. Checksums are published with each ", "release", "."),
    docs_label="Documents", d_privacy="Privacy", d_terms="Terms of use", branch=1,
)

e = html.escape
LOGO = '<svg class="logo-mark" viewBox="0 0 200 200" aria-hidden="true"><path d="M30 118V30H170V170H82" fill="none" stroke="currentColor" stroke-width="12"/><path d="M75.21 124V60.41H94.58L99.93 99.18L105.24 60.41H124.79V124H113.15V78.18L105.85 124H94.44L86.7 78.18V124Z" fill="currentColor"/></svg>'

def page(c, name, title, body, extra_js=""):
    n = c["nav"]; f = c["footer"]; l = c["lang"]
    return f'''<!doctype html>
<html lang="{l}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{e(title)}</title>
<meta name="description" content="{e(c["desc"])}">
<meta name="theme-color" content="#1b1b1b">
<meta property="og:title" content="{e(title)}">
<meta property="og:description" content="{e(c["desc"])}">
<link rel="icon" href="../assets/img/mira-icon.svg" type="image/svg+xml">
<link rel="alternate" hreflang="{c["other"]}" href="../{c["other"]}/{name}">
<link rel="preload" href="../assets/fonts/Anton-Regular.ttf" as="font" type="font/ttf" crossorigin>
<link rel="stylesheet" href="../style.css">
</head>
<body>
<a class="skip" href="#mira-contenu">{e(c["skip"])}</a>
<header class="top"><div class="wrap">
  <a class="brand" href="index.html" aria-label="MIRA">{LOGO}<span>MI<em>RA</em></span></a>
  <nav aria-label="Main">
    <a class="hide-sm" href="index.html#atouts">{e(n[0])}</a>
    <a class="hide-sm" href="index.html#comptes">{e(n[1])}</a>
    <a href="privacy.html"{' class="active" aria-current="page"' if name == "privacy.html" else ""}>{e(n[2])}</a>
  </nav>
  <a class="lang" href="../{c["other"]}/{name}" hreflang="{c["other"]}" aria-label="{e(c["switch_label"])}">{c["switch"]}</a>
  <a class="cta" href="index.html#telecharger">{icon("download")} {e(n[3])}</a>
</div></header>
<main id="mira-contenu">
{body}
</main>
<footer class="foot"><div class="wrap">
  <span>{e(f[0])}</span>
  <nav aria-label="Footer">
    <a href="privacy.html">{e(f[1])}</a>
    <a href="terms.html">{e(f[2])}</a>
    <a href="https://github.com/nnoah29/MIRA">{e(f[3])}</a>
    <a href="mailto:iam@toffanoah.com">{e(f[4])}</a>
  </nav>
</div></footer>
{extra_js}
</body>
</html>
'''

def home(c):
    strengths = "\n".join(f'      <div class="cell"><p class="label">{e(a)}</p><h3>{e(b)}</h3><p>{e(t)}</p></div>' for a, b, t in c["strengths"])
    accounts = "\n".join(f'      <li><span class="avatar" style="background:{LOOK[i][1]}">{LOOK[i][0]}</span><span><strong>{e(n)}</strong><span class="label">{e(h)}</span></span></li>' for i, (n, h) in enumerate(c["accounts"]))
    tabs = "\n".join(f'      <button type="button" role="tab" id="mira-tab-{i}" aria-selected="{"true" if i == 0 else "false"}" tabindex="{0 if i == 0 else -1}" aria-controls="mira-tab-{i}-panel">{e(t)}</button>' for i, (t, _) in enumerate(c["systems"]))
    panels = []
    for i, ((file, cmd), (_, system)) in enumerate(zip(FILES, c["systems"])):
        cmdblock = f'''
      <p class="label">{e(c["then"])}</p>
      <div class="field"><code>{e(cmd)}</code><button class="button" type="button" data-copy="{e(cmd)}">{icon("copy")} <span>{e(c["copy"])}</span></button></div>''' if cmd else ""
        panels.append(f'''    <div class="panel" id="mira-tab-{i}-panel" role="tabpanel" aria-labelledby="mira-tab-{i}">
      <p class="label">{e(system)}</p>
      <div class="field"><code>{e(file)}</code><a class="button primary" href="{RELEASE}{file}">{icon("download")} {e(c["get"])}</a></div>{cmdblock}
    </div>''')
    r = c["req"]
    return f'''<section class="hero"><div class="wrap">
  <h1><span class="line">{e(c["h1"][0])}</span> <span class="line">{e(c["h1"][1])}<span class="caret"></span></span></h1>
  <div class="intro">
    <p class="lede">{e(c["lede"])}</p>
    <div class="actions">
      <a class="button primary" href="#telecharger">{icon("download")} {e(c["dl"])}</a>
      <a class="button" href="https://github.com/nnoah29/MIRA">{icon("code")} {e(c["src"])}</a>
    </div>
    <p class="label">{e(c["tags"])}</p>
  </div>
</div></section>

<section class="shot"><div class="wrap"><figure>
  <img src="../assets/img/mira-app.webp" width="1909" height="1024" fetchpriority="high" alt="{e(c["shot_alt"])}">
</figure></div></section>

<section class="band" id="atouts"><div class="wrap">
  <h2>{e(c["f_title"])}<span class="caret"></span></h2>
  <div class="grid">
{strengths}
  </div>
</div></section>

<section class="band" id="comptes"><div class="wrap">
  <h2>{e(c["a_title"])}<span class="caret"></span></h2>
  <ul class="accounts">
{accounts}
  </ul>
</div></section>

<section class="band" id="telecharger"><div class="wrap">
  <h2>{e(c["d_title"])}<span class="caret"></span></h2>
  <div class="tabs" role="tablist" aria-label="{e(c["system_label"])}">
{tabs}
  </div>
  <div id="mira-panels">
{chr(10).join(panels)}
  </div>
  <div class="panel script">
    <p class="label accent">{e(c["install_label"])}</p>
    <div class="field"><code>{e(INSTALL)}</code><button class="button" type="button" data-copy="{e(INSTALL)}">{icon("copy")} <span>{e(c["copy"])}</span></button></div>
    <p class="note">{e(c["install_note"])}</p>
  </div>
  <p class="note">{e(r[0])}<a href="https://github.com/nnoah29/MIRA/releases/latest">{e(r[1])}</a>{r[2]}</p>
</div></section>'''

def doc(c, which):
    t = (SRC / f"mira-{which}" / f"mira-{which}.component.html").read_text()
    t = t.split("@if (i18n.lang() === 'fr') {", 1)[1]
    parts = t.split("} @else {", 1)
    chunk = parts[0] if c["branch"] == 0 else parts[1].rsplit("}", 1)[0]
    chunk = re.sub(r'<a \[routerLink\]="i18n\.privacy\(\)">', '<a href="privacy.html">', chunk)
    chunk = re.sub(r"\{\{ c\.docs\.(privacy|terms) \}\}", lambda m: c["d_" + m.group(1)], chunk)
    chunk = re.sub(r"\n {6,10}", "\n", chunk).strip()
    cur = lambda k: ' class="active" aria-current="page"' if k == which else ""
    return f'''<article class="doc"><div class="wrap">
  <nav class="doc-tabs" aria-label="{e(c["docs_label"])}">
    <a href="privacy.html"{cur("privacy")}>{e(c["d_privacy"])}</a>
    <a href="terms.html"{cur("terms")}>{e(c["d_terms"])}</a>
  </nav>
{chunk}
</div></article>'''

JS = (ROOT / "tools" / "site.js").read_text()
for c in (FR, EN):
    d = ROOT / c["lang"]; d.mkdir(exist_ok=True)
    (d / "index.html").write_text(page(c, "index.html", c["t_home"], home(c), f"<script>\n{JS.replace('__COPIED__', c['copied']).replace('__COPY__', c['copy'])}</script>"))
    (d / "privacy.html").write_text(page(c, "privacy.html", c["t_privacy"], doc(c, "privacy")))
    (d / "terms.html").write_text(page(c, "terms.html", c["t_terms"], doc(c, "terms")))
