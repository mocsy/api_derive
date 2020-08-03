import React, { FC } from 'react'
import { List, Datagrid, Edit, Create, SimpleForm, TextField, TextInput, ImageField } from 'react-admin';
import BookIcon from '@material-ui/icons/LibraryBooks';
import { ListComponentProps, FieldProps, EditComponentProps, CreateComponentProps } from '../types';

export const PostIcon = BookIcon;

export const PostList: FC<ListComponentProps> = props => (
    <List {...props}>
        <Datagrid rowClick="edit">
            <TextField source="id" />
            <TextField source="title" />
            <TextField source="content" />
            <TextField source="author" />
            <ImageField source="image" />
            {/* <NumberField source="timestamp" /> */}
        </Datagrid>
    </List>
);

const PostTitle: FC<FieldProps<any>> = ({ record }) => {
    return <span>Post {record ? `"${record.title}"` : ''}</span>;
};

export const PostEdit: FC<EditComponentProps> = (props) => (
    <Edit title={<PostTitle />} {...props}>
        <SimpleForm>
            <TextField disabled source="id" />
            <TextInput source="title" />
            <TextInput source="content" options={{ multiLine: true }} />
            <TextInput source="author" />
            <TextInput source="image" title="title" type="url" pattern="https://.*" />
            <ImageField source="image" title="title" />
            {/* <DateField disabled source="timestamp" /> */}
        </SimpleForm>
    </Edit>
);

export const PostCreate: FC<CreateComponentProps> = (props) => (
    <Create title="Create a Post" {...props}>
        <SimpleForm>
            <TextField disabled source="id" />
            <TextInput source="title" />
            <TextInput source="content" options={{ multiLine: true }} />
            <TextInput source="author" />
            <TextInput source="image" title="title" type="url" pattern="https://.*" />
            <ImageField source="image" title="title" />
            {/* <DateField disabled source="timestamp" /> */}
        </SimpleForm>
    </Create>
);
