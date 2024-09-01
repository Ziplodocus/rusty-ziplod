

pub async fn check(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), Error> {
  let data = ctx.data.read().await;

  let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

  let intros = storage_client.get_objects(msg.message.author.tag, "intro");
  let outros = storage_client.get_objects(msg.message.author.tag, "outro");

  let reply : String = "Here are your current themes... \nIntros:".to_string();

  for let theme of intros {
    reply += "\n\t- " theme.name;
  }

  reply += "\nOutros:";

  for let theme of outros {
    reply += `\n\t- ${theme}`;
  }


  msg.message.reply(reply);
  return msg;
};
