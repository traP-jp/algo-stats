#[derive(Debug, Clone)]
pub struct TrapMemberWithAcAccount {
    pub trap_account_name: String,
    pub ac_account_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrapMember {
    pub trap_account_name: String,
    pub is_active: bool,
    pub is_algo_team: bool,
    pub grade: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AcDetailedInfo {
    pub algo_rating: Vec<ContestResult>,
    pub heur_rating: Vec<ContestResult>,
}

#[derive(Debug, Clone)]
pub struct ContestResult {
    pub is_rated: bool,
    pub place: i32,
    pub old_rating: i32,
    pub new_rating: i32,
    pub diff: i32,
    pub performance: i32,
    pub contest_screen_name: String,
    pub contest_name: String,
    pub end_time: String,
}

#[derive(Debug, Clone)]
pub struct Contest {
    pub contest_name: String,
    pub contest_url: String,
    pub set_at: String,
}
