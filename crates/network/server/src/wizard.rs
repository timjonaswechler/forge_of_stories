//! # [Ratatui] Original Demo example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md
//!
//! TODO: Aufgaben:
//! - [x] Wird eine Aufgabe erledigt, bekommt man in der Kategorie-Übersicht keine aktuellen Counter, wie viele Settings schon erledigt sind.
//! - [ ] Wenn es Kategorien gibt, die Unterkategorien haben:
//!   - [ ] Im Help Panel sollen die Subkategorien aufgelistet werden wie im Category Panel
//!   - [ ] Wenn man die Help einer Subkategorie hat, sollen die Felder der Subkategorie aufgelistet werden im Help Panel
//!   - [x] Wenn man im Kategorie Panel die Kategorie auswählt mit Enter, soll die erste Unterkategorie im Kategorie Panel ausgewählt werden und man springt direkt ins Settings Panel in die erste Unterkategorie
//!   - [x] Die Darstellung der Auflistung in den Unterkategorien muss verbessert werden. Aktuell gibt es nur └ (https://www.compart.com/en/unicode/U+2514), es fehlt aber für alle anderen außer der letzten Subkategorie das Symbol ├ (https://www.compart.com/en/unicode/U+251C)
//! - [x] Ist die letzte Unterkategorie abgeschlossen, oder die Kategorie abgeschlossen und man springt aus dem Settings Panel raus in das Kategorie Panel, soll die nächste Kategorie angewählt werden.
//! - [ ] Im Settings Panel:
//!   - [x] Werte ändern durch 'e' drücken ermöglichen
//!   - [ ] Eingabe validieren:
//!     - [ ] Falls die Eingabe falsch ist, den Wert in Rot darstellen und in der Zeile darunter angeben, warum der Fehler auftritt in rotem Text
//!   - [ ] Bei Enums mit einem Popup arbeiten
//!   - [ ] Bei Arrays Möglichkeit überlegen
//!   - [x] Default-Wert in Klammern () hinter dem geänderten Wert angeben, sobald der Wert vom Default abweicht (in Grau hinterlegt)
//! - [x] Das "es kann nur bis zum nächsten noch nicht abgehakten Setting gesprungen werden" wieder entfernen (freie Bewegung soll möglich sein). Am Ende muss alles abgehakt sein.
//! - [ ] Wenn alle Settings gecheckt sind, muss irgendwie die Möglichkeit sein, dass man dann die Settings abschließt und in einem Overview-Popup nochmal alle Einstellungen auftauchen oder man gefragt wird, ob das alles ist. Vielleicht kann man auch einen speziellen Kategorie-Eintrag machen, der keine Aufzählung usw. bekommt mit der Bezeichnung "Fertig" oder "Einstellungen Bestätigen". Es wird geschaut, ob alle Einstellungen abgehakt sind, falls nicht kommt eine Fehler-Warnung mit dem Hinweis, dass noch Einstellungen nicht komplett sind. Wenn alles gut geht, wird der Wizard geschlossen und die Server Settings, die man erstellt hat, werden ausgegeben, damit man weitermachen kann und der Server mit den Einstellungen startet.
//! - [x] Wenn der Wizard vorzeitig beendet wird, soll das Programm panic auswerfen und den Prozess abbrechen. Vielleicht kann man auch einen Status ausgeben, wenn die Wizard-Run beendet wird, und man fragt, ob der Status auf "completed" steht, wenn nicht abbrechen.
//! - [ ] Struktur des Startup verbessern, wizard.rs - run() => crossterm.rs - run() => run_app()
//! - [ ] Server Settings definieren und daraus automatisch die Settings Kategorien und Sub kategorien erstellen, Defaultwerte und Auswahlmöglichkeiten
//! - [ ] im Edit- modus, werte ein haben die durch , getrennt werden werden untereinander auf gelistet bei der eingabe.  sprich wenn der user , drückt wird automatisch einen neue zeile erstellt in der der coruser dann ist. Edge case wenn man in dem Panel an den runteren Rand kommt muss automaitsch gescrollt werde.
//! - [x] im Edit- modus ein Courser einbauen.

use std::error::Error;
use std::time::Duration;

mod app;
mod crossterm;
mod ui;

pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(50);

    super::wizard::crossterm::run(tick_rate, true)?;
    Ok(())
}
