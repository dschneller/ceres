use prettytable::Table;
use prettytable::cell::Cell;
use prettytable::format;
use prettytable::row::Row;
use std::collections::HashMap;
use std::io::Write;

use provider::{InstanceDescriptor, InstanceDescriptorFields};
use output::*;

pub struct TableOutputInstances {
    pub fields: Vec<InstanceDescriptorFields>,
    pub tags_filter: Option<Vec<String>>,
}

impl Default for TableOutputInstances {
    fn default() -> Self {
        TableOutputInstances {
            fields: vec![
                InstanceDescriptorFields::InstanceId,
                InstanceDescriptorFields::InstanceType,
                InstanceDescriptorFields::State,
                InstanceDescriptorFields::PrivateIpAddress,
                InstanceDescriptorFields::PublicIpAddress,
                InstanceDescriptorFields::LaunchTime,
            ],
            tags_filter: None,
        }
    }
}

impl OutputInstances for TableOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(
            Row::new(
                self.fields.iter().map(|f| Cell::new(header_for_field(f))).collect::<Vec<_>>()
            ));

        // We have to create / allocate the Strings first since `Table` only accepts `&str` and some
        // `InstanceDescriptorFields` need to allocate representations first, e.g., `InstanceDescriptorFields::Tags`
        let mut rows = Vec::new();
        for instance in instances {
            let row = self.fields
                .iter()
                .map(|f| value_for_field(f, instance))
                .collect::<Vec<_>>();
            rows.push(row);
        }
        for r in rows {
            table.add_row(
                Row::new(
                    r.iter().map(|cell| Cell::new(cell)).collect::<Vec<_>>()
                ));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn header_for_field(field: &InstanceDescriptorFields) -> &str {
    match *field {
        InstanceDescriptorFields::BlockDeviceMapping => "Block Device Mapping",
        InstanceDescriptorFields::Hypervisor => "Hypervisor",
        InstanceDescriptorFields::InstanceId => "Instance Id",
        InstanceDescriptorFields::IamInstanceProfile => "Iam Instance Profile",
        InstanceDescriptorFields::InstanceType => "Instance Type",
        InstanceDescriptorFields::LaunchTime => "Launch Time",
        InstanceDescriptorFields::Monitoring => "Monitoring",
        InstanceDescriptorFields::Placement => "Placement",
        InstanceDescriptorFields::PrivateDnsName => "Private DNS Name",
        InstanceDescriptorFields::PrivateIpAddress => "Private IP Address",
        InstanceDescriptorFields::PublicDnsName => "Public DNS Name",
        InstanceDescriptorFields::PublicIpAddress => "Public IP Address",
        InstanceDescriptorFields::RootDeviceName => "Root Device Name",
        InstanceDescriptorFields::RootDeviceType => "Root Device Type",
        InstanceDescriptorFields::SecurityGroups => "Security Groups",
        InstanceDescriptorFields::State => "State",
        InstanceDescriptorFields::StateReason => "State Reason",
        InstanceDescriptorFields::Tags(_) => "Tags",
    }
}

fn value_for_field(field: &InstanceDescriptorFields, instance: &InstanceDescriptor) -> String {
    match *field {
        InstanceDescriptorFields::BlockDeviceMapping =>
            Some(instance.block_device_mappings.join("\n")),
        InstanceDescriptorFields::Hypervisor => instance.hypervisor.clone(),
        InstanceDescriptorFields::IamInstanceProfile => instance.iam_instance_profile.clone(),
        InstanceDescriptorFields::InstanceId => instance.instance_id.clone(),
        InstanceDescriptorFields::InstanceType => instance.instance_type.clone(),
        InstanceDescriptorFields::LaunchTime => instance.launch_time.clone(),
        InstanceDescriptorFields::Monitoring => instance.monitoring.clone(),
        InstanceDescriptorFields::Placement => instance.placement.clone(),
        InstanceDescriptorFields::PrivateDnsName => instance.private_dns_name.clone(),
        InstanceDescriptorFields::PrivateIpAddress => instance.private_ip_address.clone(),
        InstanceDescriptorFields::PublicDnsName => instance.public_dns_name.clone(),
        InstanceDescriptorFields::PublicIpAddress => instance.public_ip_address.clone(),
        InstanceDescriptorFields::RootDeviceName => instance.root_device_name.clone(),
        InstanceDescriptorFields::RootDeviceType => instance.root_device_type.clone(),
        InstanceDescriptorFields::SecurityGroups =>
            Some(instance.security_groups.join("\n")),
        InstanceDescriptorFields::State => instance.state.clone(),
        InstanceDescriptorFields::StateReason => instance.state_reason.clone(),
        InstanceDescriptorFields::Tags(ref tags_filter) => {
            Some(format_tags(instance.tags.as_ref().unwrap(), tags_filter.as_ref().map(|x| x.as_slice())))
        },
    }.unwrap_or_else(|| String::from("-"))
}

/// Format a `HashMap` of `String` -> `Option<String>` into a single line, pretty string.
fn format_tags(tags: &HashMap<String, Option<String>>, tags_filter: Option<&[String]>) -> String {
    let empty = String::from("");
    let mut concat = String::new();

    let mut keys: Vec<_> = if let Some(tags_filter) = tags_filter {
        tags
            .keys()
            .filter(|&k| tags_filter.contains(k))
            .collect()
    } else {
        tags.keys().collect()
    };
    keys.sort();
    let mut iter = keys.into_iter();

    if let Some(k) = iter.next() {
        concat.push_str(k);
        concat.push_str("=");
        concat.push_str(tags.get(k).unwrap().as_ref().unwrap_or(&empty));
    };
    for k in iter {
        concat.push_str(", ");
        concat.push_str(k);
        concat.push_str("=");
        concat.push_str(tags.get(k).unwrap().as_ref().unwrap_or(&empty));
    }
    concat
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn format_tags_empty() {
        let tags = HashMap::new();

        let res = format_tags(&tags, None);

        let expected = String::from("");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_one_kv() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));

        let res = format_tags(&tags, None);

        let expected = String::from("key1=value1");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_multiple_kv() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));
        tags.insert("key2".to_owned(), None);
        tags.insert("key3".to_owned(), Some("value3".to_owned()));

        let res = format_tags(&tags, None);

        let expected = String::from("key1=value1, key2=, key3=value3");
        assert_that(&res).is_equal_to(&expected);
    }


    #[test]
    fn format_tags_multiple_kv_with_filter() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));
        tags.insert("key2".to_owned(), None);
        tags.insert("key3".to_owned(), Some("value3".to_owned()));
        let filter: &[String] = &["key1".to_owned(), "key3".to_owned()];
        let tags_filter = Some(filter);

        let res = format_tags(&tags, tags_filter);

        let expected = String::from("key1=value1, key3=value3");
        assert_that(&res).is_equal_to(&expected);
    }
}
