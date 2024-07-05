from telegram import ForceReply, Update
from telegram.ext import Application, CommandHandler, ContextTypes, MessageHandler, filters
import os
import uuid
import pymongo

db = pymongo.MongoClient("mongodb://localhost:27017/")["vicweb"]
postsCollection = db["posts"]

def main() -> None:
    tgbot = Application.builder().token(os.getenv("TG_TOKEN")).build()
    tgbot.add_handler(MessageHandler(filters.ALL, message))

def message(update: Update, context: ContextTypes) -> None:
    if (update.message.chat.id != os.getenv("TG_CHAT_ID")):
        return
    print("New channel message recived!")
    Iid = str(uuid.uuid4())
    Mcontent = update.message.text

    dbdata = postsCollection.find_one({"content": Mcontent})

    if (dbdata == None):
        print (f"Message not found in database! Adding message to database with id: {Iid}")
        postsCollection.insert_one({"id": Iid, "content": Mcontent})
        print("Post added to database!")
        return 
    print("Message skipped (already in database)!")

    


if (__name__ == "__main__"):
    main()

