#![allow(warnings)] 
const CONTRACT_VERMOGEN: i32 = 630;
const STROOMVERBRUIK_TOPSHUIS: i32 = 68;

const BEGIN_STORM_TIJD: u64 = 4;
const EIND_STORM_TIJD: u64 = 18;
const SIM_TIJD: u64 = 24;

use std::time::*;

struct Tijd{
    tijd: u64
}

impl Tijd{
    fn new_precies(tijd:u64) -> Tijd{
        Tijd {tijd}
    }
    fn new(tijd:u64) -> Tijd{
        Tijd{tijd:(tijd*60*10)}
    }
    fn new_uren(tijd:u64) -> Tijd{
        Tijd{tijd:(tijd*60*60*10)}
    }
    fn uren(&self) -> f32{
        (self.tijd as f32 /10.0/60.0/60.0)
    }
    fn sec(&self) -> f32{
        (self.tijd as f32 /10.0)
    }
    fn min(&self) -> f32{
        (self.tijd as f32 /60.0/10.0)
    }
}

#[derive(Debug)]
enum SchuivenStatus{
    Open,
    Openen,
    Gesloten,
    Sluiten,
}

struct Schuiven{
    status: SchuivenStatus,
    beweging_tijd: Tijd,
    huidig_stroomverbruik: f32,
    stroomverbruik_kering_openen: f32,
    stroomverbruik_kering_sluiten: f32,
    schuif_positie: f32
}

impl Schuiven{
    fn new(status: SchuivenStatus, beweging_tijd: Tijd, stroomverbruik_kering_openen: i32, stroomverbruik_kering_sluiten: i32) -> Schuiven{
        Schuiven {  status, 
                    stroomverbruik_kering_openen: (stroomverbruik_kering_openen as f32 /beweging_tijd.uren()), 
                    stroomverbruik_kering_sluiten: (stroomverbruik_kering_sluiten as f32 /beweging_tijd.uren()),
                    beweging_tijd, 
                    huidig_stroomverbruik: 0.0,
                    schuif_positie: 0.0
                    }
    }
    fn schuiven_simulatie(&mut self,simulatie_stap:&Tijd,verander:bool,stroom_beschikbaar: f32) -> bool{
        if verander{
            match self.status{
                SchuivenStatus::Open => self.status = SchuivenStatus::Sluiten,
                SchuivenStatus::Openen => self.status = SchuivenStatus::Sluiten,
                SchuivenStatus::Gesloten => self.status = SchuivenStatus::Openen,
                SchuivenStatus::Sluiten => self.status = SchuivenStatus::Openen,
            }
        }
        match &self.status{
            SchuivenStatus::Open => {
                self.huidig_stroomverbruik = 0.0;
                self.schuif_positie = 0.0;},
            SchuivenStatus::Openen => {
                if stroom_beschikbaar < self.stroomverbruik_kering_openen {return false;}
                self.schuif_positie = self.schuif_positie - simulatie_stap.min().clone();
                if self.schuif_positie > 0.0{
                    self.huidig_stroomverbruik = self.stroomverbruik_kering_openen;
                }
                else{
                    self.huidig_stroomverbruik = 0.0;
                    self.status = SchuivenStatus::Open
                }
            },
            SchuivenStatus::Gesloten => {
                self.huidig_stroomverbruik = 0.0;
                self.schuif_positie = self.beweging_tijd.min();},
            SchuivenStatus::Sluiten => {
                if stroom_beschikbaar < self.stroomverbruik_kering_sluiten {return false;}
                self.schuif_positie = self.schuif_positie + simulatie_stap.min();
                if self.schuif_positie < self.beweging_tijd.min(){
                    self.huidig_stroomverbruik = self.stroomverbruik_kering_sluiten;
                }
                else{
                    self.huidig_stroomverbruik = 0.0;
                    self.status = SchuivenStatus::Gesloten
                }
            },
        }
        true
    }
    fn positie_procent(&self) -> f32{
        self.schuif_positie/self.beweging_tijd.min()
    }
}

#[derive(PartialEq)]
enum Status{
    Ingeschakeld,
    Uitgeschakeld,
}

struct Stroomgebruiker{
    status: Status,
    capaciteit: i32,
    huidig_stroomverbruik: f32
}

impl Stroomgebruiker {
    fn new(status: Status, capaciteit: i32) -> Stroomgebruiker {
        
        Stroomgebruiker { huidig_stroomverbruik: if status == Status::Ingeschakeld {capaciteit as f32} else {0.0}, status, capaciteit}
        

    }
    fn stroomgebruik_stap(&mut self,simulatie_stap:&Tijd) -> f32{
        self.huidig_stroomverbruik
    }
    fn schakel_modus(&mut self, schakel:bool){
        if schakel {self.status = Status::Ingeschakeld; self.huidig_stroomverbruik = self.capaciteit as f32} else {self.status = Status::Uitgeschakeld; self.huidig_stroomverbruik = 0.0}
    }
}

#[derive(Debug,PartialEq)]
enum BatterijStatus{
    Opladen,
    Ontladen,
    Rust,
    LaagSOC,
    StormPrepare(bool),
    Storm,
    Defect
}

struct Batterij{
    status: BatterijStatus,
    capaciteit: f32,
    opgeslagen: f32,
    oplaadsnelheid: f32,
    huidig_stroomverbruik: f32,
    huidig_stroombeschikbaar: f32
}

impl Batterij{
    fn state_of_charge(&self)-> f32{
        (self.opgeslagen/self.capaciteit) as f32
    }
    fn new(batterij_capaciteit: f32, mut batterij_opgeslagen: f32) -> Batterij{
        if batterij_capaciteit < batterij_opgeslagen {
            batterij_opgeslagen = batterij_capaciteit;
        }
        Batterij{
            capaciteit: batterij_capaciteit,
            opgeslagen: batterij_opgeslagen,
            status: BatterijStatus::Ontladen,
            oplaadsnelheid: batterij_capaciteit/2.0,
            huidig_stroomverbruik: 0.0,
            huidig_stroombeschikbaar: 0.0
        }
    }
    fn update_charge(&mut self,simulatie_stap_tijd:&Tijd,beschikbare_stroomvoorziening:&f32,kering_status:&KeringStatus) -> f32{
        if kering_status == &KeringStatus::Storm(false) && self.state_of_charge() < 1.0{
            self.status = BatterijStatus::StormPrepare(false);
            self.opgeslagen += beschikbare_stroomvoorziening * simulatie_stap_tijd.uren();
            self.huidig_stroomverbruik = -beschikbare_stroomvoorziening;
            self.huidig_stroombeschikbaar = 0.0
        }
        else if kering_status == &KeringStatus::Storm(false) && self.state_of_charge() >= 1.0{
            self.status = BatterijStatus::StormPrepare(true);
            self.huidig_stroomverbruik = 0.0;
            self.huidig_stroombeschikbaar = self.oplaadsnelheid
        }
        else if kering_status == &KeringStatus::Storm(true){
            self.status = BatterijStatus::Storm;
            self.opgeslagen += beschikbare_stroomvoorziening * simulatie_stap_tijd.uren();
            self.huidig_stroomverbruik = -beschikbare_stroomvoorziening;
            self.huidig_stroombeschikbaar = self.oplaadsnelheid
        }
        else if beschikbare_stroomvoorziening > &0.0 && self.state_of_charge() < 1.0 {
            self.status = BatterijStatus::Opladen;
            self.opgeslagen += beschikbare_stroomvoorziening * simulatie_stap_tijd.uren();
            self.huidig_stroomverbruik = -beschikbare_stroomvoorziening;
            self.huidig_stroombeschikbaar = self.oplaadsnelheid
        }
        else if beschikbare_stroomvoorziening < &0.0 && self.state_of_charge() > 0.2{
            self.status = BatterijStatus::Ontladen;
            self.opgeslagen += beschikbare_stroomvoorziening * simulatie_stap_tijd.uren();
            self.huidig_stroomverbruik = -beschikbare_stroomvoorziening;
            self.huidig_stroombeschikbaar = self.oplaadsnelheid
        }
        else if self.state_of_charge() < 0.2 {
            self.status = BatterijStatus::LaagSOC;
            self.huidig_stroombeschikbaar = 0.0
        }
        else if self.state_of_charge() > 1.0 {
            self.opgeslagen = self.capaciteit;
            self.huidig_stroomverbruik = -beschikbare_stroomvoorziening;
            self.huidig_stroombeschikbaar = self.oplaadsnelheid
        }
        else if self.state_of_charge() < 0.0 {
            self.opgeslagen = 0.0;
            self.huidig_stroomverbruik = -beschikbare_stroomvoorziening;
            self.huidig_stroombeschikbaar = 0.0
        }
        else{
            self.status = BatterijStatus::Rust;
            self.huidig_stroomverbruik = 0.0;
            self.huidig_stroombeschikbaar = self.oplaadsnelheid
        }
        self.huidig_stroomverbruik
    }
}

#[derive(Debug,PartialEq)]
enum KeringStatus{
    Normaal,
    Storm(bool)
}

struct Kering {
    status: KeringStatus,
    schuiven: Schuiven,
    batterij: Batterij,
    aggregaten: Stroomgebruiker,
    topshuis: Stroomgebruiker,
    hoofdaansluiting: Stroomgebruiker,
    stroom_vraag: f32,
    stroom_aanbod: f32
    
}

impl Kering{
    fn new( stroomverbruik_topshuis: i32,
            stroomverbruik_kering_openen: i32,
            stroomverbruik_kering_sluiten: i32,
            capaciteit_hoofdaansluiting: i32,
            batterij_capacitiet: f32,
            batterij_opgeslagen: f32,
            aggregaat_capaciteit: i32,
            kering_beweging_tijd: u64) -> Kering{
                Kering {    status: KeringStatus::Normaal,
                            schuiven: Schuiven::new(SchuivenStatus::Open, Tijd::new(kering_beweging_tijd), stroomverbruik_kering_openen, stroomverbruik_kering_sluiten),
                            batterij: Batterij::new(batterij_capacitiet, batterij_opgeslagen), 
                            aggregaten: Stroomgebruiker::new(Status::Uitgeschakeld, aggregaat_capaciteit), 
                            topshuis: Stroomgebruiker::new(Status::Ingeschakeld,stroomverbruik_topshuis), 
                            hoofdaansluiting: Stroomgebruiker::new(Status::Ingeschakeld,capaciteit_hoofdaansluiting), 
                            stroom_vraag: 0.0, stroom_aanbod: 0.0}

    }
    fn bereken_stroomvraag(&mut self) -> f32{
        self.stroom_vraag = self.topshuis.huidig_stroomverbruik +
                            self.schuiven.huidig_stroomverbruik +
                            self.batterij.huidig_stroombeschikbaar;
        self.stroom_vraag
    }
    fn bereken_stroomaanbod(&mut self) -> f32{
        self.stroom_aanbod =    self.hoofdaansluiting.huidig_stroomverbruik +
                                self.aggregaten.huidig_stroomverbruik +
                                self.batterij.huidig_stroombeschikbaar;
        self.stroom_aanbod
    }
}

struct Simulatie{
    dal_tijd: bool,
    simulatie_tijd: Tijd,
    simulatie_eind_tijd: Tijd,
    kering: Kering,
    simulatie_stap_tijd: Tijd,
    totaal_stroomgebruik_batterij: i32,
    totaal_stroomgebruik_hoofdaansluiting: i32,
    totaal_stroomgebruik_aggregaten: i32,
    storm_tijd_uur: Tijd,
    batterij_oplaadtijd: Tijd
}

impl Simulatie{
    fn new(simulatie_eind_tijd: Tijd, kering: Kering,simulatie_stap_tijd:Tijd, storm_tijd_uur: Tijd) -> Simulatie{
        Simulatie { dal_tijd: true, simulatie_tijd: Tijd { tijd: 0 }, simulatie_eind_tijd, kering, simulatie_stap_tijd,
                    totaal_stroomgebruik_batterij: 0 ,totaal_stroomgebruik_hoofdaansluiting: 0,totaal_stroomgebruik_aggregaten: 0, 
                    storm_tijd_uur,batterij_oplaadtijd:Tijd::new(0)}
    }
    fn simulatie_stap(&mut self,verander:bool) -> bool{
        self.dal_tijd = self.price_calculator();
        self.kering.bereken_stroomaanbod();
        self.kering.schuiven.schuiven_simulatie(&self.simulatie_stap_tijd, verander,self.kering.stroom_aanbod);
        self.kering.bereken_stroomvraag();
        self.kering.batterij.update_charge(&self.simulatie_stap_tijd, &(&self.kering.stroom_aanbod-&self.kering.stroom_vraag),&self.kering.status);
        self.batterij_oplaadtijd = self.bereken_oplaadtijd();
        self.storm_voorspeller();
        self.status_updater()
    }
    fn status_updater(&mut self) -> bool{
        self.simulatie_tijd.tijd += self.simulatie_stap_tijd.tijd;
        if self.simulatie_tijd.tijd > self.simulatie_eind_tijd.tijd {
            println!("Einde simulatie");
            println!("Batterij: {}% - {:?}",self.kering.batterij.state_of_charge()*100.0,self.kering.batterij.status);
            println!("Kering: {:?} - Positie: {}",self.kering.schuiven.status,self.kering.schuiven.schuif_positie);
            println!("Stroom nodig: {} - Stroom beschikbaar: {}\n",self.kering.stroom_vraag,self.kering.stroom_aanbod);
            false
        }
        else if self.simulatie_tijd.min() % 15.0 == 0.0 || self.simulatie_tijd.tijd == self.simulatie_stap_tijd.tijd {
            println!("Simulatietijd: {} min - {}:{} | {}",self.simulatie_tijd.min(),(self.simulatie_tijd.uren()%24.0) as u32,
            (self.simulatie_tijd.min()%60.0) as u32, if self.dal_tijd {"€"} else {"€€"});
            println!("Batterij: {}% - {:?}",self.kering.batterij.state_of_charge()*100.0,self.kering.batterij.status);
            println!("Kering: {:?} - Sluiting: {}%",self.kering.schuiven.status,self.kering.schuiven.positie_procent()*100.0);
            println!("Stroom beschikbaar {}kW",self.kering.stroom_aanbod- self.kering.stroom_vraag);
            println!("Topshuis modus {:?}\n",self.kering.status);
            true
        }
        else{
            true
        }
    }
    fn price_calculator(&self) -> bool{
        let simulatie_uur = self.simulatie_tijd.uren() % 24.0;
        if simulatie_uur < 6.0 || simulatie_uur > 22.0 {
            true
        }
        else{
            false
        }
    }
    fn bereken_oplaadtijd(&self) -> Tijd{
        let stroom_beschikbaar_kwh = self.kering.stroom_aanbod - self.kering.stroom_vraag;
        let oplaadtijd = Tijd::new_uren(((self.kering.batterij.capaciteit - self.kering.batterij.opgeslagen) / stroom_beschikbaar_kwh) as u64);
        oplaadtijd
    }
    fn storm_voorspeller(&mut self){
        let stroom_beschikbaar_kwh = self.kering.stroom_aanbod - self.kering.stroom_vraag;
        if self.storm_tijd_uur.uren() < self.simulatie_tijd.uren() && self.kering.status != KeringStatus::Storm(true){
            self.kering.status = KeringStatus::Storm(true);
            self.kering.hoofdaansluiting.schakel_modus(false);
            println!("{}",self.kering.hoofdaansluiting.huidig_stroomverbruik);
            self.kering.schuiven.schuiven_simulatie(&self.simulatie_stap_tijd, true, 0.0);
        }
        else if self.storm_tijd_uur.uren() - self.simulatie_tijd.uren() < self.batterij_oplaadtijd.uren() && self.kering.status != KeringStatus::Storm(true){
            self.kering.status = KeringStatus::Storm(false)
        }
        
    }
    
}

fn main() {
    let mut oosterscheldekering:Kering = Kering::new(   STROOMVERBRUIK_TOPSHUIS, 
                                                        1747, 
                                                        1971, 
                                                        CONTRACT_VERMOGEN, 
                                                        4000.0, 
                                                        2000.0, 
                                                        4000,
                                                        85);
    let mut simulatie: Simulatie = Simulatie::new(      Tijd::new_uren(SIM_TIJD), 
                                                        oosterscheldekering, 
                                                        Tijd::new_precies(1),
                                                        Tijd::new_uren(BEGIN_STORM_TIJD),);   
    while simulatie.simulatie_stap(false){}
}

