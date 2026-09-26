//! # Query report command
//!
//! Runs the RFC 6352 `addressbook-query` REPORT and renders the
//! matching cards.

use std::num::NonZeroU32;

use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum};
use io_webdav::rfc6352::{
    card::list::CarddavCardListOptions,
    filter::{
        CarddavFilter, CarddavFilterTest, CarddavMatchType, CarddavPropCond, CarddavPropFilter,
        CarddavTextMatch,
    },
};
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::CarddavClient,
    report::entries::{CarddavCardEntriesOutput, EntryRow},
};

/// Query the cards of an addressbook (RFC 6352 §8.6).
///
/// The `addressbook-query` REPORT. Without filter flags every card comes
/// back. Each filter flag adds one prop-filter, which the server
/// evaluates: `--match EMAIL contains doe --match FN starts-with john`.
/// Param-filters and anything else the flags cannot express go through
/// `report raw`.
///
/// JSON output: `{"cards": [{"id", "etag", "contents"}], "truncated"}`.
#[derive(Debug, Parser)]
pub struct CarddavReportQueryCommand {
    /// Identifier of the addressbook to query.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Match a property value (TYPE: equals, contains, starts-with,
    /// ends-with).
    #[arg(long = "match", num_args = 3, value_names = ["PROP", "TYPE", "VALUE"])]
    pub matches: Vec<String>,
    /// Match a property value, negated.
    #[arg(long = "not-match", num_args = 3, value_names = ["PROP", "TYPE", "VALUE"])]
    pub not_matches: Vec<String>,
    /// Require a property to be present.
    #[arg(long = "defined", value_name = "PROP")]
    pub defined: Vec<String>,
    /// Require a property to be absent.
    #[arg(long = "not-defined", value_name = "PROP")]
    pub not_defined: Vec<String>,
    /// How the filters combine: every one (allof) or any one (anyof).
    #[arg(long, value_name = "TEST", default_value = "allof")]
    pub test: FilterTestArg,
    /// Collation of every match, the server's `i;unicode-casemap` when
    /// omitted.
    #[arg(long, value_name = "NAME")]
    pub collation: Option<String>,
    /// The most cards the server should return.
    #[arg(long, value_name = "N")]
    pub limit: Option<NonZeroU32>,
}

impl CarddavReportQueryCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();
        let fn_color = client.account.cards_list_table_fn_color();
        let opts = CarddavCardListOptions {
            limit: self.limit,
            filter: self.filter()?,
        };
        let ok = client.list_cards(&self.addressbook_id, &opts)?;

        printer.out(CarddavCardEntriesOutput {
            preset,
            id_color,
            fn_color,
            rows: ok.cards.into_iter().map(EntryRow::from).collect(),
            truncated: ok.truncated,
        })
    }

    /// Maps the filter flags onto one prop-filter each.
    fn filter(&self) -> Result<CarddavFilter> {
        let mut props = Vec::new();

        for (args, negate) in self
            .matches
            .chunks_exact(3)
            .map(|args| (args, false))
            .chain(self.not_matches.chunks_exact(3).map(|args| (args, true)))
        {
            let [name, match_type, value] = args else {
                unreachable!("clap takes exactly three values per match");
            };

            let match_type = MatchTypeArg::from_str(match_type, true)
                .map_err(|_| {
                    anyhow!(
                        "Invalid match type `{match_type}`, expected equals, contains, starts-with or ends-with"
                    )
                })?;

            let text = CarddavTextMatch {
                value: value.clone(),
                match_type: match_type.into(),
                negate,
                collation: self.collation.clone(),
            };

            props.push(prop_filter(
                name,
                CarddavPropCond::Match {
                    texts: vec![text],
                    params: vec![],
                },
            ));
        }

        for name in &self.defined {
            let cond = CarddavPropCond::Match {
                texts: vec![],
                params: vec![],
            };
            props.push(prop_filter(name, cond));
        }

        for name in &self.not_defined {
            props.push(prop_filter(name, CarddavPropCond::IsNotDefined));
        }

        Ok(CarddavFilter {
            test: self.test.into(),
            props,
        })
    }
}

fn prop_filter(name: &str, cond: CarddavPropCond) -> CarddavPropFilter {
    CarddavPropFilter {
        name: name.to_string(),
        test: CarddavFilterTest::AnyOf,
        cond,
    }
}

/// How the prop-filters of a query combine.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FilterTestArg {
    /// Every filter matches.
    #[value(name = "allof")]
    AllOf,
    /// Any filter matches.
    #[value(name = "anyof")]
    AnyOf,
}

impl From<FilterTestArg> for CarddavFilterTest {
    fn from(arg: FilterTestArg) -> Self {
        match arg {
            FilterTestArg::AllOf => Self::AllOf,
            FilterTestArg::AnyOf => Self::AnyOf,
        }
    }
}

/// How a text match compares.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum MatchTypeArg {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
}

impl From<MatchTypeArg> for CarddavMatchType {
    fn from(arg: MatchTypeArg) -> Self {
        match arg {
            MatchTypeArg::Equals => Self::Equals,
            MatchTypeArg::Contains => Self::Contains,
            MatchTypeArg::StartsWith => Self::StartsWith,
            MatchTypeArg::EndsWith => Self::EndsWith,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use io_webdav::rfc6352::filter::{
        CarddavFilterTest, CarddavMatchType, CarddavPropCond, CarddavTextMatch,
    };

    use super::CarddavReportQueryCommand;

    fn parse(args: &[&str]) -> CarddavReportQueryCommand {
        let argv = ["query", "contacts"].iter().chain(args);
        CarddavReportQueryCommand::try_parse_from(argv).unwrap()
    }

    #[test]
    fn maps_each_flag_onto_its_own_prop_filter() {
        let cmd = parse(&[
            "--match",
            "EMAIL",
            "contains",
            "doe",
            "--not-match",
            "FN",
            "starts-with",
            "john",
            "--defined",
            "TEL",
            "--not-defined",
            "NICKNAME",
            "--test",
            "anyof",
            "--collation",
            "i;octet",
        ]);
        let filter = cmd.filter().unwrap();

        assert_eq!(filter.test, CarddavFilterTest::AnyOf);
        assert_eq!(filter.props.len(), 4);

        let names: Vec<&str> = filter.props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["EMAIL", "FN", "TEL", "NICKNAME"]);

        let CarddavPropCond::Match { texts, params } = &filter.props[1].cond else {
            panic!("expected a text match");
        };
        assert!(params.is_empty());
        assert_eq!(
            texts[0],
            CarddavTextMatch {
                value: "john".into(),
                match_type: CarddavMatchType::StartsWith,
                negate: true,
                collation: Some("i;octet".into()),
            }
        );

        let CarddavPropCond::Match { texts, params } = &filter.props[2].cond else {
            panic!("expected a presence test");
        };
        assert!(texts.is_empty() && params.is_empty());
        assert_eq!(filter.props[3].cond, CarddavPropCond::IsNotDefined);
    }

    #[test]
    fn defaults_to_a_match_all_allof() {
        let filter = parse(&[]).filter().unwrap();
        assert_eq!(filter, Default::default());
    }

    #[test]
    fn refuses_an_unknown_match_type() {
        let cmd = parse(&["--match", "EMAIL", "like", "doe"]);
        assert!(cmd.filter().unwrap_err().to_string().contains("`like`"));
    }
}
