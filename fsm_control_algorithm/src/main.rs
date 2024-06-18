use tokio::time::{sleep, Duration};

//States of Master State Machine
#[derive(Debug)]
enum MasterState {
    Initial,
    GetSmpQuality,
    ResetQuality,
    Valid,
    Questionable,
    Invalid,
    VerifyBackupMUPrincipal,
    SwitchToBackupSmp,
    ToogleMUQuality,
}

//States of the Slave State Machine 
#[derive(Debug)]
enum SlaveState {
    GetSmpValue,
    CalculusOfDispersion,
    CheckErrorPercentage,
    KeepMU,
    ToogleMUDispersion,
    ResetDispersion,
 
}

//Events MasterStateMachine and SlaveStateMachine
#[derive(Debug)]
enum Event {
    // Master Events
    ValidSmp,
    QuestionableSmp,
    InvalidSmp,
    ContInvalidLess10,
    ContInvalidMore10,
    BackupSmpValid,
    BackupSmpInvalid,
    // Slave Events
    GetSample,
    ContSMU,
    ContKMU,
    Error25Less,
    Error25MoreEqual,
}

// Methods thats be implemented in the State Machines
trait StateMachine {
    type State;
    fn new() -> Self;
    fn handle_event(&mut self, event: Option<Event>);
    fn automatic_transition(&mut self); 
    fn current_state(&self) -> &Self::State;
}

// Master State Machine, everything is regarding the Master State Machine

// Structs regarding the Master State Machine 
struct MasterStateMachine {
    state: MasterState,
    block_flag: bool, // Flag of blocking slave
    cont_invalid: usize, // Add cont_invalid field
}

// Implementation using the traits regarding State Machine implement the methods of State Machine in Master State Machine

impl StateMachine for MasterStateMachine {
    type State = MasterState;

    fn new() -> Self {
        Self {
            state: MasterState::Initial,
            block_flag: false,
            cont_invalid: 0, // Initialize cont_invalid
        }
    }

    fn handle_event(&mut self, event: Option<Event>) {
        match (&self.state, event) {

            // State Get Sample Quality  
            (MasterState::GetSmpQuality, Some(Event::ValidSmp)) => {
                println!("Master: GetSmpQuality -> Valid");
                self.state = MasterState::Valid;
            }
            (MasterState::GetSmpQuality, Some(Event::QuestionableSmp)) => {
                println!("Master: GetSmpQuality -> Questionable");
                self.state = MasterState::Questionable;
            }
            (MasterState::GetSmpQuality, Some(Event::InvalidSmp)) => {
                println!("Master: GetSmpQuality -> Invalid");
                self.state = MasterState::Invalid;
            }

            // State Invalid
            (MasterState::Invalid, Some(Event::ContInvalidLess10)) => {
                println!("Master: Invalid -> GetSmpQuality");
                self.cont_invalid += 1; // Increment cont_invalid
                self.state = MasterState::GetSmpQuality;
            }

            (MasterState::Invalid, Some(Event::ContInvalidMore10)) => {
                println!("Master: Invalid -> VerifyBackupMUPrincipal");
                self.state = MasterState::VerifyBackupMUPrincipal;
            }

            // State Verify Backup MU Principal

            (MasterState::VerifyBackupMUPrincipal, Some(Event::BackupSmpValid)) => {
                println!("Master: VerifyBackupMUPrincipal -> SwitchToBackupSmp");
                self.block_flag = true; // Set flag to block the slave state machine
                self.state = MasterState::SwitchToBackupSmp;
            }

            (MasterState::VerifyBackupMUPrincipal, Some(Event::BackupSmpInvalid)) => {
                println!("Master: VerifyBackupMUPrincipal -> ToogleMUQuality");
                self.block_flag = true; // Set flag to block the slave state machine
                self.state = MasterState::ToogleMUQuality;
            }

            _ => {
                // Default case: no state change
                println!("Master: No state change");
            }
        }
    }

    fn automatic_transition(&mut self) {
        match self.state {
            /*MasterState::Idle => {
                // Example: Automatically transition from Idle to Running
                println!("Master: Idle -> Running (automatic transition)");
                self.state = MasterState::Running;
                self.cont_invalid += 1; // Increment cont_invalid
            }*/
            // Automatic transisitions in MasterStateMachine

            MasterState::Initial => {
                println!("Master: Initial -> Get Sample Quality");
                self.state = MasterState::GetSmpQuality;
            }

            MasterState::Valid => {
                println!("Master: Valid -> Reset Quality");
                self.state = MasterState::ResetQuality;
            }

            MasterState::Questionable => {
                println!("Master: Questionable -> GetSmpQuality");
                self.state = MasterState::GetSmpQuality;
            }

            MasterState::SwitchToBackupSmp => {
                println!("Master: SwitchToBackupSmp -> ResetQuality");
                self.state = MasterState::ResetQuality;
            }

            MasterState::ToogleMUQuality => {
                println!("Master: ToogleMUQuality -> ResetQuality");
                self.state = MasterState::ResetQuality;  
            }

            MasterState::ResetQuality => {
                println!("Master: ResetQuality -> GetSmpQuality");
                self.block_flag = false;
                self.cont_invalid = 0;
                self.state = MasterState::GetSmpQuality;
            }

            _ => {
                println!(" Stay at the same state: {:?}", self.state);
            }
        }
    }

    fn current_state(&self) -> &MasterState {
        println!("Current State of Master FSM: {:?}", &self.state);
        &self.state
    }
}

impl MasterStateMachine {
    fn set_block_flag(&mut self, block: bool) {
        self.block_flag = block;
    }

    fn is_blocked(&self) -> bool {
        println!("Flag is : {}", &self.block_flag);
        self.block_flag
    }
}

// Slave State Machine, everything is regarding the Slave State Machine

// Structs regarding the Slave State Machine  
struct SlaveStateMachine {
    state: SlaveState,
    cont_smu: usize, // Add cont_SMU field
    cont_kmu: usize, // Add cont_KMU field
    error: f32, // value of the percentage of error
    n_smp: usize, // how many samples before try to change the MU
}

// Implementation using the traits regarding State Machine implement the methods of State Machine in Slave State Machine

impl StateMachine for SlaveStateMachine {
    type State = SlaveState;

    fn new() -> Self {
        Self {
            state: SlaveState::GetSmpValue,
            cont_smu: 0, // Initialize cont SMU
            cont_kmu: 0, // Initialize cont KMU
            error: 0.0,  // Error Percentage
            n_smp: 9,    // Number of samples to considering swap between the Merging Unit
        }
    }

    fn handle_event(&mut self, event: Option<Event>) {
        match (&self.state, event) {

            // State Calculus of Dispersion
            (SlaveState::CalculusOfDispersion, Some(Event::GetSample)) => {
                
                // Needs to implement the Calculus between the two samples
                /*if x > y
                    self.cont_smu += 1;
                else
                    self.cont_kmu += 1;
                */
                self.cont_smu += 1;
                if self.cont_smu > self.n_smp {
                    self.state = SlaveState::CheckErrorPercentage;
                    println!("Slave: CalculusOfDispersion -> CheckErrorPercentage;");
                }
                else if self.cont_kmu > self.n_smp{
                    self.state = SlaveState::KeepMU;
                    println!("Slave: CalculusOfDispersion -> KeepMU");
                }
                else {
                    self.state = SlaveState::GetSmpValue;
                    println!("Slave: CalculusOfDispersion -> GetSmpValue");
                }
            }
            
            // State Check the Error Percentage 
            (SlaveState::CheckErrorPercentage, Some(Event::Error25Less)) => {
                println!("Slave: CheckErrorPercentage -> KeepMU ");
                self.state = SlaveState::KeepMU;
                
            }

            (SlaveState::CheckErrorPercentage, Some(Event::Error25MoreEqual)) => {
                println!("Slave: CheckErrorPercentage -> ToogleMUDispersion ");
                self.state = SlaveState::ToogleMUDispersion;
            }

            // State 

            _ => {
                println!(" Stay at the same state: {:?}", self.state);
            }
        }
    }

    fn automatic_transition(&mut self) {
        match self.state {
            
            SlaveState::GetSmpValue => {
                println!("Slave: GetSmpValue -> CalculusOfDispersion (automatic transition)");
                self.state = SlaveState::CalculusOfDispersion;
            }

            SlaveState::ToogleMUDispersion => {
                println!("Slave: ToogleMUDispersion -> ResetDispersion (automatic transition)");
                self.state = SlaveState::ResetDispersion;
            }

            SlaveState::KeepMU => {
                println!("Slave: KeepMU -> ResetDispersion (automatic transition)");
                self.state = SlaveState::ResetDispersion;
            }

            SlaveState::ResetDispersion => {
                println!("Slave: ResetDispersion -> GetSmpValue (automatic transition)");
                self.state = SlaveState::GetSmpValue;
                self.cont_kmu = 0;
                self.cont_smu = 0;
                self.error = 0.0;
            }

            _ => {
                
            }
        }
    }

    fn current_state(&self) -> &SlaveState {
        println!("Current State of Slave FSM: {:?}", &self.state);
        &self.state
    }
}

// Implementation of methods only regarding the Slave State Machine

impl SlaveStateMachine {
    fn handle_event_with_master(&mut self, event: Option<Event>, master: &MasterStateMachine) {
        if master.is_blocked() {
            println!("Slave state machine is blocked by the master.");
            self.cont_kmu = 0;
            self.cont_smu = 0;
            self.state = SlaveState::GetSmpValue;
            return;
        }
        self.handle_event(event);
        self.automatic_transition(); // Check for automatic transitions
    }
}


// Main Thread of the State Machine
#[tokio::main]
async fn main() {
    let mut master_sm = MasterStateMachine::new();
    let mut slave_sm = SlaveStateMachine::new();

    // Debug States of the State Machines
    // Debug of Master State Machine


    //Debug State Machine Through the Invalid Sample Backup Valid
    println!("");
    println!("Debug State Machine Through the Invalid Sample Backup Valid ");
    master_sm.current_state();
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.handle_event(Some(Event::ContInvalidMore10));
    master_sm.is_blocked();
    master_sm.handle_event(Some(Event::BackupSmpValid));
    master_sm.is_blocked();
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    master_sm.current_state();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();

    // Debug State Machine Through the Invalid Sample Backup Valid
    println!("");
    println!(" Debug State Machine Through the Invalid Sample Backup Valid");
    master_sm.current_state();
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.handle_event(Some(Event::ContInvalidMore10));
    master_sm.is_blocked();
    master_sm.handle_event(Some(Event::BackupSmpInvalid));
    master_sm.is_blocked();
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    master_sm.current_state();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();

    // Debug State Machine Through the Questionable
    println!("");
    println!("Debug State Machine Through the Questionable");
    master_sm.current_state();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.current_state();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();

    // Debug State Machine Through the Valid
    println!("");
    println!("Debug State Machine Through the Valid");
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    master_sm.current_state();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();

    // Debug State Machine Through the Valid/Questionable/Invalid -> Switch to Backup Samples
    println!("");
    println!("Debug State Machine Through the Valid/Questionable/Invalid -> Switch to Backup Samples");
    master_sm.current_state();
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    master_sm.handle_event(Some(Event::InvalidSmp));
    master_sm.handle_event(Some(Event::ContInvalidLess10));
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    master_sm.handle_event(Some(Event::QuestionableSmp));
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();
    master_sm.handle_event(Some(Event::ValidSmp));
    master_sm.automatic_transition();
    master_sm.automatic_transition();
    println!("ContInvalid is: {}", master_sm.cont_invalid);
    master_sm.is_blocked();
    master_sm.current_state();

    // Debug Stay in the same state if has no event, stay at the same state


    // Debug State Machine Through the 25% more or equal of error
    println!("");
    println!("Slave State Machine");
    println!("Debug State Machine Through the 25% more or equal of error");
    slave_sm.current_state();
    slave_sm.automatic_transition();
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    println!("ContSMU is: {}", slave_sm.cont_smu);
    println!("ContKMU is: {}", slave_sm.cont_kmu);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    println!("ContSMU is: {}", slave_sm.cont_smu);
    println!("ContKMU is: {}", slave_sm.cont_kmu);
    slave_sm.current_state();
    slave_sm.handle_event_with_master(Some(Event::Error25MoreEqual), &master_sm);
    slave_sm.current_state();
    slave_sm.automatic_transition();


    // Debug State Machine Through the 25% less of error
    println!("");
    println!("Debug State Machine Through the 25% less of error");
    slave_sm.current_state();
    slave_sm.automatic_transition();
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    println!("ContSMU is: {}", slave_sm.cont_smu);
    println!("ContKMU is: {}", slave_sm.cont_kmu);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    println!("ContSMU is: {}", slave_sm.cont_smu);
    println!("ContKMU is: {}", slave_sm.cont_kmu);
    slave_sm.current_state();
    slave_sm.handle_event_with_master(Some(Event::Error25Less), &master_sm);
    slave_sm.current_state();
    slave_sm.automatic_transition();

    // Debug State Machine Through the Keep MU, change the counter to cont_kmu increasing and not cont_smu
    /*Uncomment after change the things in cont_smu to cont_kmu */
    /*
    println!("");
    println!("Debug State Machine Through the Keep MU, change the counter to cont_kmu increasing and not cont_smu");
    slave_sm.current_state();
    slave_sm.automatic_transition();
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    println!("ContSMU is: {}", slave_sm.cont_smu);
    println!("ContKMU is: {}", slave_sm.cont_kmu);
    slave_sm.handle_event_with_master(Some(Event::GetSample), &master_sm);
    println!("ContSMU is: {}", slave_sm.cont_smu);
    println!("ContKMU is: {}", slave_sm.cont_kmu);
    slave_sm.current_state();
    slave_sm.automatic_transition();
    */
    



    // Simulate events
    /*

    master_sm.set_block_flag(true); // Set block flag to true

    slave_sm.handle_event_with_master(Some(Event::FinishProcessing), &master_sm); // Slave state machine is blocked by the master

    master_sm.set_block_flag(false); // Remove block

    */
    
}
