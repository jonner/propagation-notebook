# Nice to have features
- [x] Before starting, it would be nice to know the current status of the harvest dates in the DB
- it would be nice to be able to go back if you made a mistake
- it would be nice to know the common name(s) of the taxon being looked up
- [x] it would be nice to know the common name(s) of the taxon in inaturalist when choosing which one is associated with the local taxon
- it would be nice to have some way to change the inaturalist taxon if it's saved in the db (e.g. Gentianopsis virgata ssp. virgata is current sent to the species)

# For full region lookups
- We should have an easy way to cancel out of the entire loop
- we should have a way to start part way through the list (e.g. "skip first 50")
- or only lookup certain taxa (those without dates? only natives?)
- when the dates are not updated, nothing seems to be printed, so it's not obvious to the user what happened
- we don't need the 'Preparing...' message
- we don't need the progress bar unless it can play better with stdout


# updates
We'll want to regularly do this lookup in a semi-automated way to keep things up to date, so we should plan on features that support this.

# misc
- do we want to store the number of samples used to do the calculation so that in the future we can update ones with small number of samples?
- alternately, if there is a species that have 'fruit' annotated on inaturalist well before they're ready to harvest, we'll need to go in and manually edit the start date, so we may need to note that somehow to avoid overwriting on the next inaturalist lookup 
