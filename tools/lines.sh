#!/bin/bash

# Initialiser le compteur de lignes total
total_lignes=0

# Fonction pour compter les lignes dans un fichier
compter_lignes() {
  local fichier=$1
  local nb_lignes=$(wc -l < "$fichier")
  total_lignes=$((total_lignes + nb_lignes))
}

# Parcourir récursivement tous les fichiers dans le dossier
parcourir_dossier() {
  local dossier=$1
  for item in "$dossier"/*; 
  do
    if [ -d "$item" ]; then
      # Si l'élément est un dossier, le parcourir récursivement
      parcourir_dossier "$item"
    elif [ -f "$item" ]; then
      # Si l'élément est un fichier, compter les lignes
      compter_lignes "$item"
    fi
  done
}

# Appel de la fonction pour parcourir le dossier fourni en argument
parcourir_dossier ".."

# Afficher le nombre total de lignes
echo $total_lignes
