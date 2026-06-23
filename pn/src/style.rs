use tabled::{
    grid::{
        config::ColoredConfig,
        dimension::CompleteDimension,
        records::{ExactRecords, Records},
    },
    settings::{Alignment, Modify, Style, TableOption, object::Columns},
};

pub struct ListTable;
pub struct DetailTable;

impl<R> TableOption<R, ColoredConfig, CompleteDimension> for ListTable
where
    R: Records,
{
    fn change(self, records: &mut R, cfg: &mut ColoredConfig, dimension: &mut CompleteDimension) {
        Style::empty()
            .horizontals([(1, tabled::settings::style::HorizontalLine::new('─'))])
            .change(records, cfg, dimension);
    }
}
impl<R> TableOption<R, ColoredConfig, CompleteDimension> for DetailTable
where
    R: ExactRecords + Records,
{
    fn change(self, records: &mut R, cfg: &mut ColoredConfig, dimension: &mut CompleteDimension) {
        Style::empty()
            .verticals([(1, tabled::settings::style::VerticalLine::new('│'))])
            .change(records, cfg, dimension);
        Modify::new(Columns::first())
            .with(Alignment::right())
            .change(records, cfg, dimension);
    }
}
