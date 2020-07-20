// Tests to be written here

// use crate::{Error, mock::*};
use crate::{*, Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn correct_mutations() {
    new_test_ext().execute_with(|| {
        let zeros = H256::zero();

        let subject = zeros;
        let relation = [zeros].to_vec();
        let v1 = "1".as_bytes().to_vec();
        let v2 = "2".as_bytes().to_vec();

        let md = Mutation {
            changes: [
                Change {
                    primary: subject,
                    relation: relation.clone(),
                    before: None,
                    after: Some(v1.clone())
                }
            ].to_vec()
        };

        assert_ok!(AnimoModule::modify(Origin::signed(1), md));
        assert_eq!(AnimoModule::animo_store(subject, relation.clone()), Some(v1.clone()));

        let md = Mutation {
            changes: [
                Change {
                    primary: subject,
                    relation: relation.clone(),
                    before: Some(v1.clone()),
                    after: Some(v2.clone())
                }
            ].to_vec()
        };

        assert_ok!(AnimoModule::modify(Origin::signed(1), md));
        assert_eq!(AnimoModule::animo_store(subject, relation.clone()), Some(v2.clone()));

        let md = Mutation {
            changes: [
                Change {
                    primary: subject,
                    relation: relation.clone(),
                    before: Some(v2.clone()),
                    after: None
                }
            ].to_vec()
        };

        assert_ok!(AnimoModule::modify(Origin::signed(1), md));
        assert_eq!(AnimoModule::animo_store(subject, relation.clone()), None);
    });
}

#[test]
fn correct_errors() {
	new_test_ext().execute_with(|| {
        let zeros = H256::zero();
        let ones = H256::from([1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]);

        let subject = zeros;
        let relation = [zeros].to_vec();
        let v1 = "1".as_bytes().to_vec();

        let md = Mutation { changes: [].to_vec() };
		assert_noop!(
			AnimoModule::modify(Origin::signed(1), md),
			Error::<Test>::EmptyChanges
		);

        let md = Mutation { changes: [
            Change {
                primary: subject,
                relation: [].to_vec(),
                before: None,
                after: Some(v1.clone())
            }
        ].to_vec() };
        assert_noop!(
			AnimoModule::modify(Origin::signed(1), md),
			Error::<Test>::EmptyRelations
		);

        let md = Mutation { changes: [
            Change {
                primary: subject,
                relation: relation.clone(),
                before: None,
                after: None
            }
        ].to_vec() };
        assert_noop!(
			AnimoModule::modify(Origin::signed(1), md),
			Error::<Test>::BeforeAndAfterStatesAreEqual
		);

        let md = Mutation { changes: [
            Change {
                primary: subject,
                relation: relation.clone(),
                before: Some(v1.clone()),
                after: None
            }
        ].to_vec() };
        assert_noop!(
			AnimoModule::modify(Origin::signed(1), md),
			Error::<Test>::BeforeStateMismatch
		);

        let md = Mutation { changes: [
            Change {
                primary: subject,
                relation: [ones, zeros].to_vec(),
                before: Some(v1.clone()),
                after: None
            }
        ].to_vec() };
        assert_noop!(
			AnimoModule::modify(Origin::signed(1), md),
			Error::<Test>::RelationsIsNotOrdered
		);
	});
}
