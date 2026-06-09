use tabled::{
    grid::{
        config::ColoredConfig,
        dimension::CompleteDimension,
        records::{ExactRecords, Records},
    },
    settings::{Alignment, Modify, Style, TableOption, object::Columns},
};

pub struct BasicTable;

impl<R> TableOption<R, ColoredConfig, CompleteDimension> for BasicTable
where
    R: Records,
{
    fn change(self, records: &mut R, cfg: &mut ColoredConfig, dimension: &mut CompleteDimension) {
        Style::empty().change(records, cfg, dimension);
    }
}
pub struct DetailTable;
impl<R> TableOption<R, ColoredConfig, CompleteDimension> for DetailTable
where
    R: ExactRecords + Records,
{
    fn change(self, records: &mut R, cfg: &mut ColoredConfig, dimension: &mut CompleteDimension) {
        BasicTable.change(records, cfg, dimension);
        Modify::new(Columns::first())
            .with(Alignment::right())
            .change(records, cfg, dimension);
    }
}
