<div align="center">

<img src="data/icons/logo.png" alt="Hardware Monitor for COSMIC" width="160">

# Hardware Monitor for COSMIC™

**La température de votre machine, d'un coup d'œil dans le panneau.**

[![licence](https://img.shields.io/badge/licence-MIT-blue?style=flat-square)](LICENSE)
[![version](https://img.shields.io/github/v/tag/Kai-J-G/cosmic-ext-hardware-monitor?style=flat-square&label=version)](https://github.com/Kai-J-G/cosmic-ext-hardware-monitor/tags)
[![écrit en Rust](https://img.shields.io/badge/écrit%20en-Rust-000000?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![pour le bureau COSMIC](https://img.shields.io/badge/pour%20le%20bureau-COSMIC%E2%84%A2-6b21a8?style=flat-square)](https://system76.com/cosmic)

[🇬🇧](README.md) · [🇫🇷](README.fr.md)

<img src="data/screenshots/overview.png" alt="La vue d'ensemble, avec les cadrans processeur, GPU, mémoire et disque" width="420">

</div>

---

## Installation

Il vous faut une session **COSMIC™**. Le reste est couvert ci-dessous.

<details open>
<summary><strong>Arch · CachyOS · EndeavourOS</strong></summary>

```bash
sudo pacman -S --needed rustup just git base-devel
```

```bash
git clone https://github.com/Kai-J-G/cosmic-ext-hardware-monitor.git
cd cosmic-ext-hardware-monitor
makepkg -si
```

Cela construit un vrai paquet, que `pacman` suit ensuite normalement.

</details>

<details>
<summary><strong>Fedora</strong></summary>

```bash
sudo dnf install cargo just git libxkbcommon-devel pkgconf-pkg-config pciutils
```

```bash
git clone https://github.com/Kai-J-G/cosmic-ext-hardware-monitor.git
cd cosmic-ext-hardware-monitor
just install
```

</details>

<details>
<summary><strong>Debian · Ubuntu</strong></summary>

```bash
sudo apt install cargo just git libxkbcommon-dev pkg-config pciutils
```

Si `just` n'est pas disponible, utilisez `cargo install just`.

```bash
git clone https://github.com/Kai-J-G/cosmic-ext-hardware-monitor.git
cd cosmic-ext-hardware-monitor
just install
```

</details>

> **La première compilation prend quelques minutes.** Elle télécharge et compile
> la boîte à outils COSMIC. Les suivantes prennent quelques secondes.
> `just install` place tout dans votre dossier personnel et ne demande jamais
> `sudo`.

### Ajout au panneau

1. Ouvrez **Paramètres → Bureau → Panneau → Applets**
2. Cliquez sur **Ajouter une applet**
3. Choisissez **Hardware Monitor for COSMIC™**

Rien n'apparaît ? Déconnectez-vous puis reconnectez-vous : le panneau ne
cherche les nouvelles applets qu'à son démarrage.

---

## Ce que vous obtenez

Un thermomètre dans votre panneau qui change de couleur quand ça chauffe, et,
à un clic, le tableau complet.

<div align="center">
<img src="data/screenshots/cpu.png" alt="Températures, charge et fréquences par cœur" width="430">
</div>

- **Processeur** — température, charge, fréquence, consommation, et chaque cœur
  avec sa barre et sa fréquence en direct
- **GPU** — charge, fréquence, consommation, températures bord et point chaud, VRAM
- **Mémoire** — utilisée, disponible, fichier d'échange, avec un historique
- **Stockage** — vitesses de lecture et d'écriture, espace libre par partition
- **En un coup d'œil** — débit réseau, charges moyennes, températures des
  disques, temps de fonctionnement

Les couleurs suivent l'accent de votre bureau. Les températures gardent leur
propre échelle : un processeur chaud a toujours l'air chaud.

### Paramètres

Cliquez sur l'engrenage dans la fenêtre. Vous pouvez suivre le thème du bureau
ou le fixer clair ou sombre, basculer entre °C et °F, et choisir la fréquence
d'actualisation (1 à 5 secondes).

---

## Mise à jour et suppression

**Mettre à jour** — récupérez et réinstallez de la même façon :

```bash
git pull && makepkg -si    # Arch
git pull && just install   # ailleurs
```

Vos réglages sont conservés. Redémarrez ensuite le panneau avec
`pkill cosmic-panel`, il revient aussitôt.

**Supprimer** — retirez d'abord l'applet du panneau, dans **Paramètres → Bureau
→ Panneau → Applets**, puis :

```bash
sudo pacman -R cosmic-ext-hardware-monitor   # Arch
just uninstall                               # ailleurs
```

---

## Un problème ?

<details>
<summary><strong>La température du processeur affiche exactement 40 °C</strong></summary>

C'est la valeur affichée quand aucun capteur n'est trouvé. Voyez ce que votre
système expose :

```bash
for d in /sys/class/hwmon/hwmon*; do echo "$(basename $d): $(cat $d/name)"; done
```

Si vous ne voyez ni `k10temp` (AMD) ni `coretemp` (Intel), chargez le module
avec `sudo modprobe k10temp` ou `sudo modprobe coretemp`.

</details>

<details>
<summary><strong>La consommation affiche « Estimation… »</strong></summary>

Certaines distributions réservent le compteur d'énergie du noyau à root :
l'applet ne peut alors pas le lire. Tout le reste fonctionne. Pour vérifier :

```bash
ls -l /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
```

Des droits `-r--------` signifient qu'il est réservé à root.

</details>

<details>
<summary><strong>Aucun GPU détecté</strong></summary>

- **AMD** — nécessite le pilote `amdgpu` chargé
- **NVIDIA** — nécessite `nvidia-smi`, fourni avec le pilote propriétaire.
  Nouveau ne remonte rien.
- **Intel** — les graphiques intégrés ne remontent qu'une température

Le GPU est détecté une fois au démarrage : redémarrez l'applet après avoir
installé un pilote.

</details>

<details>
<summary><strong>Pas de température pour un disque SATA</strong></summary>

Les disques NVMe fonctionnent d'office. Les disques SATA nécessitent un module :

```bash
sudo modprobe drivetemp
```

Ajoutez `drivetemp` à `/etc/modules-load.d/` pour qu'il persiste au redémarrage.

</details>

<details>
<summary><strong>Mon GPU porte un nom étrange</strong></summary>

Le nom vient du pilote, avec `lspci` en repli. Installez `pciutils` s'il manque.
Certaines cartes partagent un identifiant entre plusieurs modèles : le nom peut
donc tous les énumérer.

</details>

---

## Pour les développeurs

<details>
<summary><strong>Compiler et modifier</strong></summary>

| Commande | Effet |
| --- | --- |
| `just build-release` | Compilation optimisée |
| `just build-debug` | Compilation rapide, non optimisée |
| `just run` | Lance l'applet directement |
| `just check` | `cargo check` et `cargo test` |
| `just install` / `just uninstall` | Installe dans `~/.local`, ou retire |

Les relevés viennent directement de `/sys` et `/proc` — aucun démon, aucun
service auxiliaire, aucune dépendance à `lm_sensors`. `/sys/class/hwmon` est
parcouru une seule fois par actualisation et partagé entre les collecteurs
processeur, GPU et stockage ; ce qui ne change pas (modèle du processeur,
fabricant du GPU) est résolu une fois au démarrage.

```
src/
  main.rs       point d'entrée, et un aperçu de l'articulation
  app.rs        état, messages, boucle de mise à jour
  config.rs     réglages persistants
  i18n.rs       localisation, et la macro fl!()
  hardware/     lit la machine
  views/        dessine la fenêtre
i18n/           traductions, un fichier Fluent par langue
data/           entrée de bureau, métadonnées, icônes, captures
```

`cargo doc --open` est le chemin d'entrée le plus rapide.

</details>

<details>
<summary><strong>Traduire</strong></summary>

Toutes les chaînes sont dans `i18n/en/cosmic_ext_hardware_monitor.ftl`. Pour
ajouter une langue, copiez ce dossier et traduisez les valeurs :

```bash
mkdir -p i18n/de && cp i18n/en/*.ftl i18n/de/
```

Gardez les identifiants à gauche des `=` et les `{ $variables }` tels quels.
L'applet choisit la langue selon les réglages de votre bureau. `cargo test`
vérifie qu'aucune traduction n'oublie une entrée.

Le français est inclus comme exemple.

</details>

<details>
<summary><strong>Flatpak</strong></summary>

```bash
flatpak-builder --user --install --force-clean build io.github.kai_j_g.CosmicHardwareMonitor.json
```

Deux relevés ne peuvent pas fonctionner dans le bac à sable et sont masqués
plutôt qu'affichés faux : le nombre de processus et l'utilisation par partition.
NVIDIA n'est pas pris en charge dans la version Flatpak, `nvidia-smi` n'étant
pas dans l'environnement d'exécution.

</details>

---

## Crédits

Inspiré par [TempTyle](https://github.com/neojakey/TempTyle) de neojakey.

## Assistance par IA

Une partie de ce projet a été écrite avec l'aide de
[Claude](https://claude.ai) (Anthropic) : une refonte du code source, le système
de localisation, la gestion du bac à sable Flatpak, les métadonnées
d'empaquetage et l'essentiel de cette documentation. L'implémentation initiale
de l'applet et ses orientations de conception sont les miennes, et les commits
assistés par IA portent une ligne `Co-Authored-By`.

## Marque déposée

COSMIC™ est une marque de [System76, Inc.](https://system76.com) Cette applet
tierce et non officielle est destinée **au bureau COSMIC™** ; elle n'est ni
affiliée à System76 ni approuvée par lui. Elle respecte la
[politique de marque COSMIC](https://github.com/pop-os/cosmic-epoch/blob/master/TRADEMARK.md) :
espace de noms `cosmic-ext-` recommandé, et son propre espace d'App ID plutôt
que les préfixes réservés `cosmic-` ou `com.system76.`.

## Licence

[MIT](LICENSE)
