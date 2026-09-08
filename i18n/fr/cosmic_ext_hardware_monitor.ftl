# Traduction française.

app-title = Moniteur matériel
back-to-overview = ← Vue d'ensemble

## Vue d'ensemble

cpu = Processeur
gpu = Carte graphique
memory = Mémoire
disk = Disque

network = Réseau
upload = Envoi
download = Réception

cpu-usage = Utilisation du processeur
usage-user = utilisateur
usage-system = système
usage-idle = inactif

load-average = Charge moyenne
load-1m = 1 min
load-5m = 5 min
load-15m = 15 min

gpu-usage = Utilisation du GPU
gpu-memory-used = Mémoire utilisée
gpu-memory-size = Mémoire totale
gpu-absent = Graphiques intégrés ou GPU au repos

storage = Stockage
uptime = Temps de fonctionnement ›

## Processeur

temperature = Température
load-utilization = Charge / Utilisation
clock-speed = Fréquence
cpu-temperature-curve = Courbe de température du processeur

power-draw = Consommation
power-draw-label = Consommation :
power-estimating = Estimation…
processes-label = Processus :
threads-label = Fils d'exécution :
handles-label = Descripteurs :

all-cores = Les { $count } cœurs

## Carte graphique

graphics = Graphiques
gpu-unavailable = Aucun GPU dédié détecté ou métriques indisponibles.
utilization = Utilisation
not-available = N/D
gpu-temperature-curve = Courbe de température du GPU
gpu-junction = Jonction : { $temperature }
vram-usage = Utilisation de la VRAM : { $percent } %
vram-used-of-total = { $used } / { $total }

## Mémoire

memory-title = Mémoire et fichier d'échange
memory-in-use = Utilisée
memory-available = Disponible
memory-percent-of-ram = { $percent } % de la RAM
memory-percent-free = { $percent } % libre
memory-history = Historique d'utilisation de la RAM
physical-memory = Mémoire physique : { $percent } %
memory-in-use-of-total = { $used } utilisés / { $total } au total
memory-committed = Engagée
memory-cached = En cache
swap-used = Échange utilisé
swap-available = Échange disponible
swap-percent-used = { $percent } % utilisés
swap-of-total = sur { $total }

## Stockage

storage-title = Stockage et partitions
read-speed = Vitesse de lecture
write-speed = Vitesse d'écriture
partitions-title = Partitions montées et espace libre
partitions-none = Aucune partition montée détectée.
partitions-sandboxed =
    L'utilisation des partitions est indisponible dans la version Flatpak, qui ne
    peut pas voir les systèmes de fichiers de l'hôte. Les débits ci-dessus restent exacts.
partition-summary = { $percent } %  ·  { $free } libres sur { $total }

## Paramètres

settings = Paramètres

theme-title = Adaptation du thème
theme-description =
    Choisissez si le Moniteur matériel suit le thème de votre bureau ou reste fixé
    sur une apparence claire ou sombre.
theme-system = Système / Suivre le thème du bureau
theme-system-description = Bascule automatiquement entre clair et sombre selon COSMIC
theme-dark = Noir (toujours sombre)
theme-dark-description = Fond très sombre avec des accents émeraude et cyan contrastés
theme-light = Blanc (toujours clair)
theme-light-description = Fond clair avec une typographie sombre et nette

unit-title = Unité de température
unit-celsius = Celsius (°C)
unit-fahrenheit = Fahrenheit (°F)

interval-title = Intervalle d'actualisation
interval-1s = 1 s (rapide)
interval-2s = 2 s (normal)
interval-3s = 3 s
interval-5s = 5 s (économe)

## Unités

unit-megabytes = { $value } Mo
unit-gigabytes = { $value } Go
unit-kb-per-second = { $value } Ko/s
unit-mb-per-second = { $value } Mo/s
unit-watts = { $value } W
unit-megahertz = { $value } MHz
unit-gigahertz = { $value } GHz
unit-percent = { $value } %

uptime-minutes = { $minutes } minutes
uptime-hours = { $hours } heures, { $minutes } minutes
uptime-days = { $days } j, { $hours } h, { $minutes } min
