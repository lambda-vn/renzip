# Archivo

Fichiers : `archivo-latin.woff2`, `archivo-latin-ext.woff2` (variable, graisses 400 a 800).

Origine : Google Fonts, famille Archivo v25, sous-ensembles `latin` et `latin-ext`.
Auteur : Omnibus-Type. Licence : SIL Open Font License 1.1.

La police est embarquee plutot que chargee depuis fonts.googleapis.com : une
application de bureau doit s'afficher hors ligne, et la politique de securite
de contenu declaree dans `src-tauri/tauri.conf.json` n'autorise aucune origine
distante.

Pour les mettre a jour, relever les URL servies par :

    curl -H "User-Agent: Mozilla/5.0 ... Chrome/120" \
      "https://fonts.googleapis.com/css2?family=Archivo:wght@400..800&display=swap"
