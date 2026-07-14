# Propagation Notebook

`propagation-notebook` (or `pn`) is an experimental software tool designed to help native plant propagators, seed collectors, and restoration ecologists easily look up harvesting, cleaning, and propagation information about native plant species.

---

> [!WARNING]
> ### Highly Experimental Software
> **Propagation Notebook is (probably) not for you.** 
> It is still in a very early state and is highly unlikely to be useful to the casual user.
> * **No Graphical Interface:** All interaction takes place in your terminal.
> * **Manual Setup Required:** You must download external databases (like SQLite taxonomy dumps) and manipulate raw data files (like GeoJSON and YAML) to use it.
> * **Unstable Schema & APIs:** The command structure, database layout, and configuration parameters are subject to breaking changes without warning.

---

## What is the goal?

With the increase in popularity of native plants in recent years, there is a much better understanding about how to propagate and germinate many of these plant species than there was just 20 years ago. However, there are still large gaps in our knowledge about many species. Many restoration professionals have developed protocols for some of the more difficult species, but the information can be difficult to find. Often, this knowledge is passed by word of mouth between groups doing restoration in a certain area. It would be nice to have a central reference to consult.

The goal of this project is to become that central reference so that if you are involved in ecological restoration and want to grow a species for use in restoration in your region, you can look it up in propagation-notebook and find information about harvesting, cleaning, and germinating the seeds. The end goal is to make all of this information available on a website so that it is accessible from anywhere in the world. However, at the moment, only the basic infrastructure for managing the database is in place.

The tool currently covers four main areas:

### 1. Taxonomy Database (`pn taxa`)
`pn` currently uses the **Integrated Taxonomic Information System (ITIS)** as the basis of its database. You can search for species by their scientific name, synonyms, or common names, ensuring everything you track is tied to a valid taxonomic record.

### 3. Regional Ecology Profiling (`pn regions`)
Plants behave differently depending on where they are. You can define geographic boundaries (by importing standard **GeoJSON** files) to represent your state, county, or local nature reserve. For each region, you can track its origin (whether a plant is native or introduced) and several other conservation-related properties.

### 3. Seed Harvest Dates (via iNaturalist)
One of the most powerful features of Propagation Notebook is its ability to estimate seed ripening windows. 
If you assign a GeoJSON boundary to a region, `pn` can query the **iNaturalist API** for observations of specific plants recorded *inside* your region's coordinates. By filtering for observations annotated with fruiting phenology (or looking at seasonal observation patterns), it calculates a typical **Day-of-Year window** for when ripe seeds are most likely to be available in your area.

### 4. Collection & Propagation Protocols (`pn propagation`)
Growers can catalog step-by-step procedures for:
* **Collecting:** Recording ripening indicators (e.g., "pods turn brown and papery"), optimal storage conditions, and expected viability/storage life.
* **Cleaning:** Standardizing methods to separate seeds from chaff (e.g., winnowing, threshing, flotation) and tracking literature references/citations.
* **Propagation:** Defining multi-step germination protocols (e.g., cold moist stratification for 60 days, followed by warmth) and recording confidence scores for each method.

---

## CLI Command Overview

All interactions are done using the `pn` binary. By default, it prints clean text tables, but you can request structured outputs for scripting using the global `--fmt` flag (e.g., `pn --fmt json` or `pn --fmt yaml`).

### General Commands
* `pn init`  
  Initializes or updates the local SQLite database.

### Taxonomy Commands (`pn taxa`)
* `pn taxa search <query>`  
  Searches the imported database for a plant by scientific name, synonym, or common name.
* `pn taxa show <taxon-name-or-id>`  
  Displays detailed information about a taxon, including its botanical hierarchy, synonyms, regional status, notes, and propagation history.
* `pn taxa import <sqlite-db-path>`  
  Imports taxonomic records from a downloaded ITIS SQLite database.
* `pn taxa link <taxon-name-or-id> [search | set | clear]`  
  Establishes or manages links between your local database records and iNaturalist IDs.
* `pn taxa [collecting | cleaning | propagation | notes] <taxon-name-or-id> <subcommand>`  
  Manages seed collection metadata, cleaning steps, germination protocols, and free-form notes specifically associated with that plant.

### Region Commands (`pn regions`)
* `pn regions list`  
  Lists all defined geographic regions in your database.
* `pn regions add <name> --geometry-file <geojson-path>`  
  Creates a new region and imports its boundary geometry from a GeoJSON file.
* `pn regions lookup-harvest-dates <id>`  
  Runs a bulk query against iNaturalist for all species in the region to estimate seed collection windows. Use `--interactive` to review each species' data before saving.
* `pn regions taxa <region-id> list [--native] [--ready-to-harvest]`  
  Lists the plants associated with a region. You can filter down to show only native species, or species **ready to harvest today** based on calculated windows.

### Global Propagation Protocols
* `pn propagation [list | show | add | edit | remove]`  
  Manages reusable, general templates for seed pre-treatment, germination, and seedling establishment (e.g., "60-Day Cold Moist Stratification").
