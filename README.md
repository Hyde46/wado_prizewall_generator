## Wado card prizewall generator
Consumes a `.csv` with the following fields:
```
name,condition,set,language,eur,rwp,wp,display
```
Generates "Prizewall" images of these cards by querying and then fetching the image from `scryfall.com` and placing them neatly on a page.
If too many images are in the `.csv`, more pages will be generated.

Where
- `name` is the name of the card with which Scryfall will be queried
- `condition` of the card
- `set` of the card ( Optional )
- `language` of the card ( Defaults to english )
- `eur` euro price of the card
- `rwp` Real wado points price of the card ( `eur / 2.5` )
- `wp` Final wado price point amount of the card
- `display` is the alternative name of a card to display. Uses `display` instead of `name` if set (`Swords to plowshares -> StP`)


## Build 
Install rust and then run
```
cargo build
```


## Run and generate prizewall
```
cargo run <path_to_csv>
```

The resulting pricewall pngs will be located at `./output`

## Example output 
Result will look something like this
![Example usage](output/prizewall_p2.png)
